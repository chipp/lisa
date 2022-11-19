mod response;
use elisheba::Token;
use response::Response;

use crate::{
    discover::discover,
    message::{Header, Message},
    Result,
};

use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};

use log::{debug, error, trace};

use serde::Serialize;
use serde_json::{json, Value};
use tokio::{
    net::UdpSocket,
    time::{error::Elapsed, timeout, Duration},
};

pub struct Device {
    command_id: u16,
    addr: Addr,
    token: Token<16>,
}

struct Addr {
    ip: Ipv4Addr,
    port: u16,
}

impl From<&Addr> for SocketAddr {
    fn from(addr: &Addr) -> Self {
        SocketAddr::V4(std::net::SocketAddrV4::new(addr.ip, addr.port))
    }
}

impl Device {
    pub fn new(ip: Ipv4Addr, token: Token<16>) -> Device {
        Device {
            command_id: 1,
            addr: Addr { ip, port: 54321 },
            token,
        }
    }

    pub async fn handshake(&mut self) -> Result<Header> {
        discover(Some(self.addr.ip)).await
    }

    pub async fn send<P: Serialize>(&mut self, command: &str, params: P) -> Result<Value> {
        let addr = SocketAddr::from(&self.addr);

        let header = self.handshake().await?;
        let handshake_ts = Instant::now();

        loop {
            let now = Instant::now();
            let seconds_since_handshake = now.duration_since(handshake_ts).as_secs() as u32;

            let send_ts = header.ts + seconds_since_handshake;

            trace!("sending command {} with id {}", command, self.command_id);

            let json = prepare_send_params(self.command_id, command, &params);

            let data = serde_json::to_vec(&json)?;
            let message = Message::encode(data, self.token, header.id, send_ts)?;

            match send_message(message, addr).await {
                Ok(message) => {
                    let data = message.decode(self.token)?;
                    let response: Response = serde_json::from_slice(&data)?;

                    self.set_command_id(response.id() + 1);
                    debug!("next command id {}", self.command_id);

                    return match response {
                        Response::Ok { id: _, result } => Ok(result),
                        Response::Err { id: _, error } => {
                            error!("{:?}", error);
                            Err(Box::new(error))
                        }
                    };
                }
                Err(err) => match err.downcast::<Elapsed>() {
                    Ok(_) => {
                        self.set_command_id(self.command_id + 10);
                        error!("retrying with command_id {}", self.command_id)
                    }
                    Err(err) => return Err(err),
                },
            };
        }
    }

    fn set_command_id(&mut self, command_id: u16) {
        if command_id <= 999 {
            self.command_id = command_id;
        } else {
            self.command_id = 1;
        }
    }
}

async fn send_message(message: Message, addr: SocketAddr) -> Result<Message> {
    let mut bytes = vec![];
    bytes.resize_with(message.len(), Default::default);
    message.write_to(&mut bytes);

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(addr).await?;

    socket.send(&bytes).await?;

    trace!("{} send command", addr);

    loop {
        trace!("{} waiting for response", addr);

        let mut buffer = vec![0; 1024];

        let result = timeout(Duration::from_secs(5), socket.recv_from(&mut buffer)).await?;
        let (size, _) = result?;

        if size > 0 {
            trace!("{} received response of size {}", addr, size);

            let message = Message::read_from(&buffer[..size]);
            trace!("{} parsed message {:?}", addr, message);

            return Ok(message);
        }
    }
}

fn prepare_send_params<P: Serialize>(command_id: u16, command: &str, params: &P) -> Value {
    json!({
        "id": command_id,
        "method": command,
        "params": params
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct SomeCommand<'a> {
        some_key: &'a str,
    }

    #[test]
    fn test_send_params() {
        let params = prepare_send_params(
            1,
            "some_command",
            &SomeCommand {
                some_key: "some_value",
            },
        );

        assert_eq!(
            params,
            json!({
                "id": 1,
                "method": "some_command",
                "params": {
                    "some_key": "some_value"
                }
            })
        )
    }
}
