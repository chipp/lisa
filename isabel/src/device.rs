use crate::{
    discover::discover,
    message::{Header, Message},
    Result, Token,
};

use std::net::{Ipv4Addr, SocketAddr};

use log::trace;
use serde_json::Value;
use tokio::{
    net::UdpSocket,
    time::{timeout, Duration},
};

pub struct Device {
    command_id: u16,
    addr: Addr,
    token: Token,
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
    pub fn new(ip: [u8; 4], token: Token) -> Device {
        Device {
            command_id: 0,
            addr: Addr {
                ip: ip.into(),
                port: 54321,
            },
            token,
        }
    }

    pub async fn handshake(&mut self) -> Result<Header> {
        discover(Some(self.addr.ip)).await
    }

    pub async fn send(&mut self, command: &str, params: Vec<Value>) -> Result<Value> {
        let header = self.handshake().await?;
        let send_ts = header.ts + 1;

        self.command_id += 1;
        let json = serde_json::json!({
            "id": self.command_id,
            "method": command,
            "params": params
        });

        let data = serde_json::to_vec(&json)?;
        let message = Message::encode(data, self.token, header.id, send_ts)?;

        let mut bytes = vec![];
        bytes.resize_with(message.len(), Default::default);
        message.write_to(&mut bytes);

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let addr = SocketAddr::from(&self.addr);
        socket.connect(addr).await?;

        socket.send(&bytes).await?;

        trace!("{} send command {}", addr, command);

        loop {
            trace!("{} waiting for response", addr);

            let mut buffer = vec![0; 1024];
            let result = timeout(Duration::from_secs(5), socket.recv_from(&mut buffer)).await?;

            let (size, _) = result?;

            if size > 0 {
                trace!("{} received response of size {}", addr, size);

                let message = Message::read_from(&buffer[..size]);

                trace!("{} parsed message {:?}", addr, message);

                let data = message.decode(self.token)?;
                trace!(
                    "{} decoded payload {}",
                    addr,
                    std::str::from_utf8(&data).unwrap_or("<invalid utf-8 string>")
                );

                let payload = serde_json::from_slice::<serde_json::Value>(&data)?;
                return Ok(payload);
            }
        }
    }
}
