mod error;
pub use error::Error;

mod parser;
use parser::{parse_meta, parse_packet, ParsedPacket};

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use crypto::Token;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio_util::udp::UdpFramed;

use crate::decoder::DnsCoder;
use crate::devices::{SonoffDevice, SonoffDevicesManager};

const MDNS_IP: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(MDNS_IP, 5353));

type SocketItem = (Vec<u8>, SocketAddr);
type SocketWriter = SplitSink<UdpFramed<DnsCoder>, SocketItem>;
type SocketReader = SplitStream<UdpFramed<DnsCoder>>;

enum HandleResult {
    Ignored(&'static str),
    AddedNewDevice(String, SonoffDevice),
    UpdatedDevice(SonoffDevice),
}

pub struct Client {
    known_devices: HashMap<String, Token<16>>,
    manager: SonoffDevicesManager,
    write: Arc<Mutex<SocketWriter>>,
    read: Arc<Mutex<SocketReader>>,
}

impl Client {
    pub async fn connect(known_devices: HashMap<String, Token<16>>) -> std::io::Result<Self> {
        let (write, read) = Self::inner_connect().await?;

        Ok(Self {
            known_devices,
            manager: SonoffDevicesManager::default(),
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
        })
    }

    async fn inner_connect() -> std::io::Result<(SocketWriter, SocketReader)> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;

        let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 5353);
        socket.bind(&SockAddr::from(address))?;
        socket.join_multicast_v4(&MDNS_IP, address.ip())?;

        let udp_socket = UdpSocket::from_std(socket.into())?;

        let framed = UdpFramed::new(udp_socket, DnsCoder);
        Ok(framed.split())
    }

    pub async fn reconnect(&mut self) -> std::io::Result<()> {
        let (write, read) = Self::inner_connect().await?;
        self.write = Arc::new(Mutex::new(write));
        self.read = Arc::new(Mutex::new(read));

        Ok(())
    }

    pub async fn discover(&self) -> Result<(), Error> {
        let mut write = self.write.lock().await;

        write
            .send((create_discovery_packet("_ewelink._tcp.local"), MDNS_ADDR))
            .await?;

        Ok(())
    }

    pub async fn read(&mut self) -> Result<SonoffDevice, Error> {
        let read = self.read.clone();
        let mut read = read.lock().await;

        while let Some(Ok((packet, source))) = read.next().await {
            match self.handle_packet(&packet, source)? {
                HandleResult::Ignored(_) => continue,
                HandleResult::AddedNewDevice(hostname, device) => {
                    self.query(&hostname).await?;
                    return Ok(device);
                }
                HandleResult::UpdatedDevice(device) => return Ok(device),
            }
        }

        Err(Error::Disconnected)
    }

    async fn query(&self, hostname: &str) -> Result<(), Error> {
        let mut write = self.write.lock().await;

        write
            .send((create_query_packet(hostname), MDNS_ADDR))
            .await?;

        Ok(())
    }

    fn handle_packet(&mut self, packet: &[u8], source: SocketAddr) -> Result<HandleResult, Error> {
        let _source = if let SocketAddr::V4(source) = source {
            source
        } else {
            return Ok(HandleResult::Ignored("IPv6 not supported"));
        };

        let packet = dns_parser::Packet::parse(packet)?;

        if packet.header.query {
            return Ok(HandleResult::Ignored("Query packet"));
        }

        let ParsedPacket {
            ipv4,
            info,
            service,
            host,
            port,
        } = parse_packet(&packet)?;

        let hostname = host.ok_or(Error::MissingHostname)?;
        let hostname = hostname.to_string();

        let result = if let Some(device) = self.manager.devices.get_mut(&hostname) {
            if let Some(ipv4) = ipv4 {
                device.addr.set_ip(ipv4);
            }

            if let Some(port) = port {
                device.addr.set_port(port);
            }

            if let Some(mut info) = info {
                let key = self
                    .known_devices
                    .get(&device.id)
                    .ok_or(Error::UnknownDevice(device.id.clone()))?;

                let meta = parse_meta(&mut info, *key)?;
                device.meta = meta;
            }

            HandleResult::UpdatedDevice(device.clone())
        } else {
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
                (None, None) => return Err(Error::MissingAddr).into(),
            };

            let device_type = info.get("type").ok_or(Error::MissingInfoField("type"))?;
            if device_type != "plug" {
                return Ok(HandleResult::Ignored("Unknown device type"));
            }

            let key = self
                .known_devices
                .get(&id)
                .ok_or(Error::UnknownDevice(id.clone()))?;

            let meta = parse_meta(&mut info, *key)?;

            let device = SonoffDevice { id, addr, meta };
            self.manager
                .devices
                .insert(hostname.clone(), device.clone());

            HandleResult::AddedNewDevice(hostname, device)
        };

        Ok(result)
    }
}

fn create_discovery_packet(name: &str) -> Vec<u8> {
    let mut builder = dns_parser::Builder::new_query(0, false);

    builder.add_question(
        name,
        false,
        dns_parser::QueryType::PTR,
        dns_parser::QueryClass::IN,
    );

    builder.build().unwrap()
}

fn create_query_packet(name: &str) -> Vec<u8> {
    let mut builder = dns_parser::Builder::new_query(0, false);

    builder.add_question(
        name,
        false,
        dns_parser::QueryType::A,
        dns_parser::QueryClass::IN,
    );

    builder.build().unwrap()
}
