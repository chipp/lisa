use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::time::Instant;

use super::{
    create_discovery_packet, create_query_packet, Client, HandleResult, MDNS_ADDR, SERVICE,
};
use crate::{Error, SonoffDevice};

use log::{debug, info, trace};

impl Client {
    pub async fn discover(&mut self) -> Result<Vec<SonoffDevice>, Error> {
        let socket = Self::inner_connect().await?;
        let discover = create_discovery_packet(SERVICE);
        socket.send_to(&discover, MDNS_ADDR).await?;

        let mut ids = self.keys.keys().map(Clone::clone).collect::<HashSet<_>>();
        let mut hosts = HashSet::new();

        info!("{:?} devices to discover", ids);

        let mut last_update = Instant::now();
        let mut buffer = [0; 1024];

        loop {
            let (size, addr) = socket.recv_from(&mut buffer).await?;

            match self.handle_packet(&buffer[..size], addr).await {
                Ok(HandleResult::AddedNewDevice(host, device)) => {
                    debug!("found new device: {}", device.id);
                    let query = create_query_packet(&host);
                    socket.send_to(&query, MDNS_ADDR).await?;

                    debug!("query device: {}", host);
                    hosts.insert(host);

                    last_update = Instant::now();
                }
                Ok(HandleResult::UpdatedDevice(host, device)) => {
                    debug!("updated device: {}", device.id);

                    if device.addr.ip() != &Ipv4Addr::UNSPECIFIED {
                        ids.remove(&device.id);
                        hosts.remove(&host);

                        if ids.is_empty() {
                            break;
                        }

                        info!("{} left to discover", ids.len());
                        info!("{} left to resolve", hosts.len());

                        last_update = Instant::now();
                    }
                }
                Ok(HandleResult::Ignored(msg)) => {
                    trace!("ignored {msg}");

                    if last_update.elapsed().as_secs() > 2 {
                        for host in hosts.iter() {
                            debug!("query device again: {}", host);
                            let query = create_query_packet(&host);
                            socket.send_to(&query, MDNS_ADDR).await?;
                        }

                        last_update = Instant::now();
                    }
                }
                Err(err) => trace!("error {err}"),
            }
        }

        let manager = self.manager.lock().await;
        Ok(manager.devices.values().map(Clone::clone).collect())
    }
}
