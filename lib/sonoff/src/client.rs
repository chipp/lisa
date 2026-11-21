mod discovery;
mod error;

use dns_parser::{QueryClass, QueryType};
pub use error::Error;

mod parser;
use log::{debug, trace};
use parser::{parse_meta, parse_packet, ParsedPacket};

mod request;
use request::RequestBody;

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use crypto::Token;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio_util::udp::UdpFramed;

use crate::decoder::DnsCoder;
use crate::devices::{SonoffDevice, SonoffDevicesManager};

const MDNS_IP: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(MDNS_IP, 5353));
const SERVICE: &str = "_ewelink._tcp.local";

type SocketItem = (Vec<u8>, SocketAddr);
type SocketWriter = SplitSink<UdpFramed<DnsCoder>, SocketItem>;
type SocketReader = SplitStream<UdpFramed<DnsCoder>>;

enum HandleResult {
    Ignored(&'static str),
    AddedNewDevice(String, SonoffDevice),
    UpdatedDevice(String, SonoffDevice),
}

#[derive(Clone)]
pub struct Client {
    keys: HashMap<String, Token<16>>,
    manager: Arc<Mutex<SonoffDevicesManager>>,
    write: Arc<Mutex<SocketWriter>>,
    read: Arc<Mutex<SocketReader>>,
    http_client: reqwest::Client,
}

impl Client {
    pub async fn connect(keys: HashMap<String, Token<16>>) -> std::io::Result<Self> {
        let socket = Self::inner_connect().await?;
        let framed = UdpFramed::new(socket, DnsCoder);
        let (write, read) = framed.split();

        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("HTTP client");

        Ok(Self {
            keys,
            manager: Arc::new(Mutex::new(SonoffDevicesManager::default())),
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
            http_client,
        })
    }

    async fn inner_connect() -> std::io::Result<UdpSocket> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;

        let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 5353);
        socket.bind(&SockAddr::from(address))?;
        socket.join_multicast_v4(&MDNS_IP, address.ip())?;

        UdpSocket::from_std(socket.into())
    }

    pub async fn reconnect(&mut self) -> std::io::Result<()> {
        let socket = Self::inner_connect().await?;
        let framed = UdpFramed::new(socket, DnsCoder);
        let (write, read) = framed.split();

        self.write = Arc::new(Mutex::new(write));
        self.read = Arc::new(Mutex::new(read));

        Ok(())
    }

    pub async fn read(&mut self) -> Result<SonoffDevice, Error> {
        let read = self.read.clone();
        let mut read = read.lock().await;

        while let Some(Ok((packet, source))) = read.next().await {
            match self.handle_packet(&packet, source).await? {
                HandleResult::Ignored(msg) => {
                    trace!("ignored {msg}");
                }
                HandleResult::AddedNewDevice(_, device) => {
                    debug!("found new device: {device:?}");
                    return Ok(device);
                }
                HandleResult::UpdatedDevice(_, device) => {
                    debug!("updated device: {device:?}");
                    return Ok(device);
                }
            }
        }

        Err(Error::Disconnected)
    }

    pub async fn get_state(&mut self, device_id: &str) -> Result<SonoffDevice, Error> {
        let expected = {
            let manager = self.manager.lock().await;
            manager
                .devices
                .iter()
                .find_map(|(k, v)| if v.id == device_id { Some(k) } else { None })
                .ok_or(Error::UnknownDevice(device_id.to_string()))?
                .clone()
        };

        let socket = Self::inner_connect().await?;
        let query = create_discovery_packet(SERVICE);
        socket.send_to(&query, MDNS_ADDR).await?;

        loop {
            let mut buffer = [0; 1024];
            let (size, addr) = socket.recv_from(&mut buffer).await?;
            let packet = &buffer[..size];

            match self.handle_packet(packet, addr).await {
                Ok(HandleResult::AddedNewDevice(host, device))
                | Ok(HandleResult::UpdatedDevice(host, device))
                    if host == expected =>
                {
                    return Ok(device.clone());
                }
                Ok(HandleResult::Ignored(msg)) => {
                    trace!("ignored {msg}");
                }
                Err(err) => trace!("error {err}"),
                _ => (),
            }
        }
    }

    pub async fn update_state(&self, device_id: &str, is_enabled: bool) -> Result<(), Error> {
        let key = self
            .keys
            .get(device_id)
            .ok_or(Error::UnknownDevice(device_id.to_string()))?;

        let manager = self.manager.lock().await;
        let ip = manager
            .devices
            .iter()
            .find_map(|(_, v)| {
                if v.id == device_id {
                    Some(v.addr)
                } else {
                    None
                }
            })
            .ok_or(Error::UnknownDevice(device_id.to_string()))?;

        let body = RequestBody::new(is_enabled, device_id, *key);

        let url = format!("http://{}/zeroconf/switches", ip);
        send_switch_request(&self.http_client, &url, &body).await?;

        Ok(())
    }

    async fn handle_packet(
        &mut self,
        packet: &[u8],
        source: SocketAddr,
    ) -> Result<HandleResult, Error> {
        let _source = if let SocketAddr::V4(source) = source {
            source
        } else {
            return Ok(HandleResult::Ignored("IPv6 not supported"));
        };

        let packet = dns_parser::Packet::parse(packet)?;

        if packet.header.query {
            trace!("query packet {packet:?}");
            return Ok(HandleResult::Ignored("Query packet"));
        }

        let ParsedPacket {
            ipv4,
            info,
            service,
            host,
            port,
        } = parse_packet(&packet)?;

        let host = host.ok_or(Error::MissingHostname)?;
        let host = host.to_string();

        debug!("got host {host}");

        let mut manager = self.manager.lock().await;

        let result = if let Some(device) = manager.devices.get_mut(&host) {
            debug!("host exists {host}");

            if let Some(ipv4) = ipv4 {
                debug!("updating ipv4 {ipv4}");
                device.addr.set_ip(ipv4);
            }

            if let Some(port) = port {
                debug!("updating port {port}");
                device.addr.set_port(port);
            }

            if let Some(mut info) = info {
                debug!("updating meta {info:?}");

                let key = self
                    .keys
                    .get(&device.id)
                    .ok_or(Error::UnknownDevice(device.id.clone()))?;

                let meta = parse_meta(&mut info, *key)?;
                device.meta = meta;
            }

            HandleResult::UpdatedDevice(host, device.clone())
        } else {
            debug!("new host {host}");

            let service = service.ok_or(Error::MissingService)?;
            if service != "_ewelink._tcp.local" {
                return Ok(HandleResult::Ignored("Unknown service"));
            }

            let mut info = info.ok_or(Error::MissingInfo)?;
            let id = info.remove("id").ok_or(Error::MissingInfoField("id"))?;

            let addr = match (ipv4, port) {
                (Some(ipv4), Some(port)) => SocketAddrV4::new(ipv4, port),
                (None, Some(port)) => SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port),
                (Some(ipv4), None) => SocketAddrV4::new(ipv4, 0),
                (None, None) => return Err(Error::MissingAddr),
            };

            let device_type = info.get("type").ok_or(Error::MissingInfoField("type"))?;
            if device_type != "plug" {
                return Ok(HandleResult::Ignored("Unknown device type"));
            }

            let key = self.keys.get(&id).ok_or(Error::UnknownDevice(id.clone()))?;

            let meta = parse_meta(&mut info, *key)?;

            let device = SonoffDevice { id, addr, meta };
            manager.devices.insert(host.clone(), device.clone());

            HandleResult::AddedNewDevice(host, device)
        };

        Ok(result)
    }
}

async fn send_switch_request<T: serde::Serialize>(
    client: &reqwest::Client,
    url: &str,
    body: &T,
) -> Result<(), Error> {
    let json = serde_json::to_vec(body)?;
    trace!("request: {}", String::from_utf8_lossy(&json));
    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json)
        .send()
        .await?;
    let body = response.bytes().await?;
    trace!("response: {}", String::from_utf8_lossy(&body));
    Ok(())
}

fn create_discovery_packet(service_name: &str) -> Vec<u8> {
    let mut builder = dns_parser::Builder::new_query(0, false);
    builder.add_question(service_name, false, QueryType::PTR, QueryClass::IN);
    builder.build().unwrap()
}

fn create_query_packet(host: &str) -> Vec<u8> {
    let mut builder = dns_parser::Builder::new_query(0, false);
    builder.add_question(host, false, QueryType::A, QueryClass::IN);
    builder.build().unwrap()
}

#[cfg(test)]
mod http_tests {
    use super::*;
    fn server(status: &str, body: &str) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nLocation: /redirected\r\nConnection: close\r\n\r\n{body}", body.len());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 1024];
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buf[..n]);
                if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..pos]);
                    let len = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= pos + 4 + len {
                        break;
                    }
                }
            }
            stream.write_all(response.as_bytes()).unwrap();
            String::from_utf8(bytes).unwrap()
        });
        (format!("http://{address}"), handle)
    }

    #[tokio::test]
    async fn sends_json_and_accepts_error_status() {
        let (base, handle) = server("500 Internal Server Error", "failure");
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        send_switch_request(
            &client,
            &format!("{base}/zeroconf/switches"),
            &serde_json::json!({"switch": "on"}),
        )
        .await
        .unwrap();
        let request = handle.join().unwrap().to_lowercase();
        assert!(request.starts_with("post /zeroconf/switches http/1.1\r\n"));
        assert!(request.contains("content-type: application/json"));
        assert!(request.ends_with("\r\n\r\n{\"switch\":\"on\"}"));
    }
}
