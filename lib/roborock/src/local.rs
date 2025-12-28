use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::time::Duration;

use log::{debug, info};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::protocol::{
    decode_rpc_response, LocalCodec, LocalProtocolVersion, MessageProtocol, RoborockMessage,
    RpcRequest,
};
use crate::{Error, Result};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_PORT: u16 = 58867;
const READ_TIMEOUT: Duration = Duration::from_secs(15);
#[allow(dead_code)]
const PING_INTERVAL: Duration = Duration::from_secs(10);

pub struct LocalConnection<IO> {
    stream: IO,
    codec: LocalCodec,
    buffer: Vec<u8>,
    pending: VecDeque<RoborockMessage>,
    protocol_version: LocalProtocolVersion,
}

pub type TcpLocalConnection = LocalConnection<TcpStream>;

impl LocalConnection<TcpStream> {
    pub async fn connect(
        ip: Ipv4Addr,
        local_key: String,
        connect_nonce: u32,
        hello_seq: u32,
        hello_random: u32,
    ) -> Result<Self> {
        let addr = (ip, DEFAULT_PORT);
        let stream = TcpStream::connect(addr).await?;
        Self::connect_with_stream(stream, local_key, connect_nonce, hello_seq, hello_random).await
    }
}

impl<IO> LocalConnection<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn connect_with_stream(
        stream: IO,
        local_key: String,
        connect_nonce: u32,
        hello_seq: u32,
        hello_random: u32,
    ) -> Result<Self> {
        let codec = LocalCodec::new(local_key, connect_nonce, None);

        let mut connection = Self {
            stream,
            codec,
            buffer: Vec::new(),
            pending: VecDeque::new(),
            protocol_version: LocalProtocolVersion::L01,
        };

        debug!("roborock local connect: trying L01 hello");
        let ack_nonce = connection.hello(hello_seq, hello_random).await?;
        debug!(
            "roborock local connect: L01 hello ok, ack_nonce={}",
            ack_nonce
        );
        connection.codec = connection.codec.with_ack_nonce(ack_nonce);
        connection.protocol_version = LocalProtocolVersion::L01;

        Ok(connection)
    }

    pub fn protocol_version(&self) -> LocalProtocolVersion {
        self.protocol_version
    }

    pub async fn send_rpc(
        &mut self,
        request_id: u32,
        seq: u32,
        random: u32,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let request = RpcRequest::new(request_id, method, params);
        let payload = request.to_payload()?;
        let payload_log = String::from_utf8_lossy(&payload).to_string();
        let message = RoborockMessage::new(
            self.protocol_version,
            MessageProtocol::GeneralRequest,
            seq,
            random,
            Some(payload),
        );

        debug!(
            "roborock rpc send: method={}, request_id={}, payload={}",
            method, request_id, payload_log,
        );
        self.send_message(message).await?;

        loop {
            let response = self.next_message().await?;
            if response.protocol != MessageProtocol::GeneralResponse
                && response.protocol != MessageProtocol::GeneralRequest
                && response.protocol != MessageProtocol::RpcResponse
            {
                debug!("roborock rpc skip: protocol={:?}", response.protocol);
                continue;
            }
            let payload = match response.payload {
                Some(payload) => payload,
                None => {
                    debug!(
                        "roborock rpc skip: empty payload, protocol={:?}",
                        response.protocol
                    );
                    continue;
                }
            };
            let rpc_response = decode_rpc_response(&payload)?;
            if rpc_response.id == Some(request_id) {
                debug!("roborock rpc match: request_id={}", request_id);
                debug!("roborock rpc result: {}", rpc_response.result);
                if let Some(error) = rpc_response.error {
                    return Err(error.into());
                }
                return Ok(rpc_response.result);
            }
            debug!(
                "roborock rpc ignore: response_id={:?}, request_id={}",
                rpc_response.id, request_id
            );
        }
    }

    #[allow(dead_code)]
    pub async fn ping(&mut self, seq: u32, random: u32) -> Result<()> {
        let message = RoborockMessage::new(
            self.protocol_version,
            MessageProtocol::PingRequest,
            seq,
            random,
            None,
        );
        let seq = message.seq;
        self.send_message(message).await?;

        loop {
            let response = self.next_message().await?;
            if response.protocol == MessageProtocol::PingResponse && response.seq == seq {
                return Ok(());
            }
        }
    }

    #[allow(dead_code)]
    pub async fn keep_alive_loop(
        &mut self,
        mut seq_counter: impl FnMut() -> u32,
        mut random_counter: impl FnMut() -> u32,
    ) -> Result<()> {
        loop {
            tokio::time::sleep(PING_INTERVAL).await;
            if let Err(err) = self.ping(seq_counter(), random_counter()).await {
                debug!("ping failed: {}", err);
            }
        }
    }

    async fn hello(&mut self, seq: u32, random: u32) -> Result<u32> {
        let message = RoborockMessage {
            version: LocalProtocolVersion::L01,
            seq,
            random,
            timestamp: unix_timestamp(),
            protocol: MessageProtocol::HelloRequest,
            payload: None,
        };
        let seq = message.seq;
        debug!("roborock hello send: version=L01, seq={}", seq);
        self.send_message(message).await?;

        loop {
            let response = self.next_message().await?;
            if response.protocol == MessageProtocol::HelloResponse && response.seq == seq {
                debug!("roborock hello recv: version=L01, seq={}", seq);
                info!("connected with protocol L01");
                return Ok(response.random);
            }
            debug!(
                "roborock hello skip: protocol={:?}, seq={}, expect_seq={}",
                response.protocol, response.seq, seq
            );
        }
    }

    async fn send_message(&mut self, message: RoborockMessage) -> Result<()> {
        let payload = self.codec.build_message(&message)?;
        debug!(
            "roborock send: protocol={:?}, bytes={}",
            message.protocol,
            payload.len()
        );
        self.stream.write_all(&payload).await?;
        Ok(())
    }

    async fn next_message(&mut self) -> Result<RoborockMessage> {
        if let Some(message) = self.pending.pop_front() {
            return Ok(message);
        }

        loop {
            let mut chunk = [0u8; 1024];
            let read = timeout(READ_TIMEOUT, self.stream.read(&mut chunk)).await??;
            if read == 0 {
                return Err(Error::ConnectionClosed);
            }

            self.buffer.extend_from_slice(&chunk[..read]);
            debug!(
                "roborock recv: bytes={}, buffer={}",
                read,
                self.buffer.len()
            );
            let mut decoded = self.codec.decode_messages(&mut self.buffer)?;
            if !decoded.is_empty() {
                debug!("roborock decoded: messages={}", decoded.len());
                let first = decoded.remove(0);
                for msg in decoded {
                    self.pending.push_back(msg);
                }
                return Ok(first);
            }
        }
    }
}

fn unix_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::io::duplex;

    async fn read_next(
        codec: &LocalCodec,
        server: &mut (impl AsyncRead + Unpin),
        buffer: &mut Vec<u8>,
    ) -> RoborockMessage {
        loop {
            let mut chunk = [0u8; 1024];
            let read = server.read(&mut chunk).await.unwrap();
            assert!(read > 0);
            buffer.extend_from_slice(&chunk[..read]);
            let mut decoded = codec.decode_messages(buffer).unwrap();
            if let Some(message) = decoded.pop() {
                return message;
            }
        }
    }

    #[tokio::test]
    async fn test_connect_l01_hello() {
        let local_key = "0123456789abcdef".to_string();
        let connect_nonce = 11111;
        let ack_nonce = 22222;
        let hello_seq = 100;
        let hello_random = 200;
        let (client, mut server) = duplex(4096);

        let server_key = local_key.clone();
        let server_task = tokio::spawn(async move {
            let codec = LocalCodec::new(server_key, connect_nonce, None);
            let mut buffer = Vec::new();
            let request = read_next(&codec, &mut server, &mut buffer).await;
            assert_eq!(request.protocol, MessageProtocol::HelloRequest);
            assert_eq!(request.seq, hello_seq);
            assert_eq!(request.random, hello_random);

            let response = RoborockMessage {
                version: request.version,
                seq: request.seq,
                random: ack_nonce,
                timestamp: request.timestamp,
                protocol: MessageProtocol::HelloResponse,
                payload: None,
            };
            let frame = codec.build_message(&response).unwrap();
            server.write_all(&frame).await.unwrap();
        });

        let connection = LocalConnection::connect_with_stream(
            client,
            local_key,
            connect_nonce,
            hello_seq,
            hello_random,
        )
        .await
        .unwrap();
        assert_eq!(connection.protocol_version(), LocalProtocolVersion::L01);
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_rpc_roundtrip() {
        let local_key = "0123456789abcdef".to_string();
        let connect_nonce = 12345;
        let ack_nonce = 33333;
        let hello_seq = 1;
        let hello_random = 2;
        let request_id = 999;
        let seq = 10;
        let random = 20;
        let (client, mut server) = duplex(4096);

        let server_key = local_key.clone();
        let server_task = tokio::spawn(async move {
            let mut codec = LocalCodec::new(server_key, connect_nonce, None);
            let mut buffer = Vec::new();

            let hello = read_next(&codec, &mut server, &mut buffer).await;
            assert_eq!(hello.protocol, MessageProtocol::HelloRequest);
            let response = RoborockMessage {
                version: hello.version,
                seq: hello.seq,
                random: ack_nonce,
                timestamp: hello.timestamp,
                protocol: MessageProtocol::HelloResponse,
                payload: None,
            };
            let frame = codec.build_message(&response).unwrap();
            server.write_all(&frame).await.unwrap();
            codec = codec.with_ack_nonce(ack_nonce);

            let request = read_next(&codec, &mut server, &mut buffer).await;
            assert_eq!(request.protocol, MessageProtocol::GeneralRequest);
            let payload = request.payload.expect("payload");
            let outer: serde_json::Value = serde_json::from_slice(&payload).unwrap();
            let inner_str = outer["dps"]["101"].as_str().unwrap();
            let inner: serde_json::Value = serde_json::from_str(inner_str).unwrap();
            assert_eq!(inner["id"].as_u64(), Some(request_id as u64));

            let inner_response = json!({ "id": request_id, "result": { "ok": true } });
            let outer_response =
                json!({ "dps": { "102": serde_json::to_string(&inner_response).unwrap() } });
            let response_payload = serde_json::to_vec(&outer_response).unwrap();

            let response = RoborockMessage {
                version: request.version,
                seq: request.seq,
                random: ack_nonce,
                timestamp: request.timestamp,
                protocol: MessageProtocol::GeneralResponse,
                payload: Some(response_payload),
            };
            let frame = codec.build_message(&response).unwrap();
            server.write_all(&frame).await.unwrap();
        });

        let mut connection = LocalConnection::connect_with_stream(
            client,
            local_key,
            connect_nonce,
            hello_seq,
            hello_random,
        )
        .await
        .unwrap();
        let response = connection
            .send_rpc(request_id, seq, random, "get_status", serde_json::json!([]))
            .await
            .unwrap();
        assert_eq!(response["ok"], true);
        server_task.await.unwrap();
    }
}
