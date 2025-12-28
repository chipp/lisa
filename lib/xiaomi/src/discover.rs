use std::net::{Ipv4Addr, SocketAddr};

use log::{debug, trace};
use tokio::{
    net::UdpSocket,
    time::{self, Duration},
};

use crate::message::Header;
use crate::{Error, Result};

const fn hello_bytes() -> [u8; 32] {
    let mut bytes = [0xff; 32];

    bytes[0] = 0x21;
    bytes[1] = 0x31;
    bytes[2] = 0x00;
    bytes[3] = 0x20;

    bytes
}

const HELLO_BYTES: [u8; 32] = hello_bytes();

pub async fn discover(ip: Option<Ipv4Addr>) -> Result<Header> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let ip = ip.unwrap_or(Ipv4Addr::BROADCAST);
    let addr = SocketAddr::new(ip.into(), 54321);

    socket.send_to(&HELLO_BYTES, &addr).await?;
    trace!("sent hello {}", addr);

    loop {
        let mut buffer = [0; 32];

        match time::timeout(Duration::from_secs(5), socket.recv_from(&mut buffer)).await {
            Ok(result) => {
                let (size, addr) = result?;

                if size == 32 {
                    let header = Header::read_from(&buffer);

                    debug!("ip: {}", addr.ip());
                    debug!("device id: {:x}", header.id);
                    debug!("timestamp: {}", header.ts);

                    return Ok(header);
                }
            }
            Err(_) => return Err(Error::DevicesNotFound(ip)),
        };
    }
}
