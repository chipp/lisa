use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use log::{debug, trace};
use serde_json::Value;
use tokio::net::UdpSocket;
use tokio::time;

use crate::client::DeviceConfig;
use crate::error::{Error, Result};

const DISCOVERY_PORTS: [u16; 2] = [6666, 6667];
const CONTROL_PORT: u16 = 6668;
const DEFAULT_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(15);
const PREFIX_55AA: u32 = 0x0000_55aa;
const UDP_KEY: [u8; 16] = [
    0x6c, 0x1e, 0xc8, 0xe2, 0xbb, 0x9b, 0xb5, 0x9a, 0xb5, 0x0b, 0x0d, 0xaf, 0x64, 0x9b, 0x41, 0x0a,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredDevice {
    pub device_id: String,
    pub addr: SocketAddr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDevice {
    pub config: DeviceConfig,
    pub addr: SocketAddr,
}

pub async fn resolve_devices(configs: Vec<DeviceConfig>) -> Result<Vec<ResolvedDevice>> {
    debug!(
        "resolving tuya devices: [{}]",
        configs
            .iter()
            .map(|config| format!("{}/{}", config.room, config.device_id))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let discovered = discover(discovery_timeout()).await?;
    debug!(
        "tuya discovery found {} device(s): [{}]",
        discovered.len(),
        discovered
            .iter()
            .map(|device| format!("{}@{}", device.device_id, device.addr))
            .collect::<Vec<_>>()
            .join(", ")
    );
    resolve_devices_from_discovered(configs, discovered)
}

pub fn resolve_devices_from_discovered(
    configs: Vec<DeviceConfig>,
    discovered: Vec<DiscoveredDevice>,
) -> Result<Vec<ResolvedDevice>> {
    let discovered = discovered
        .into_iter()
        .map(|device| (device.device_id.clone(), device))
        .collect::<HashMap<_, _>>();

    let mut unresolved = Vec::new();
    let mut resolved = Vec::new();

    for config in configs {
        if let Some(device) = discovered.get(&config.device_id) {
            debug!(
                "resolved tuya device {}/{} from discovery at {}",
                config.room, config.device_id, device.addr
            );
            resolved.push(ResolvedDevice {
                config,
                addr: device.addr,
            });
            continue;
        }

        if let Some(host) = config.host.as_deref() {
            let addr = SocketAddr::new(host.parse()?, CONTROL_PORT);
            debug!(
                "resolved tuya device {}/{} from configured fallback host {}",
                config.room, config.device_id, addr
            );
            resolved.push(ResolvedDevice { config, addr });
            continue;
        }

        debug!(
            "tuya device {}/{} was not discovered and has no fallback host",
            config.room, config.device_id
        );
        unresolved.push(format!("{}/{}", config.room, config.device_id));
    }

    if unresolved.is_empty() {
        Ok(resolved)
    } else {
        Err(Error::UnresolvedDevices(unresolved))
    }
}

async fn discover(timeout: Duration) -> Result<Vec<DiscoveredDevice>> {
    debug!("starting tuya UDP discovery for {:?}", timeout);
    let socket_6666 = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 6666)).await?;
    socket_6666.set_broadcast(true)?;
    let socket_6667 = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 6667)).await?;
    socket_6667.set_broadcast(true)?;

    for port in DISCOVERY_PORTS {
        let _ = socket_6666
            .send_to(&[0; 0], SocketAddrV4::new(Ipv4Addr::BROADCAST, port))
            .await;
        trace!("sent tuya discovery ping from UDP 6666 to broadcast port {port}");
        let _ = socket_6667
            .send_to(&[0; 0], SocketAddrV4::new(Ipv4Addr::BROADCAST, port))
            .await;
        trace!("sent tuya discovery ping from UDP 6667 to broadcast port {port}");
    }

    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let deadline = time::Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        match time::timeout(remaining, recv_from_any(&socket_6666, &socket_6667)).await {
            Ok(Ok((packet, addr))) => {
                trace!(
                    "received tuya UDP candidate from {} ({} bytes)",
                    addr,
                    packet.len()
                );
                match parse_discovery_packet(&packet, addr) {
                    Some(device) if seen.insert(device.device_id.clone()) => {
                        debug!(
                            "discovered tuya device {} at {} from UDP {}",
                            device.device_id, device.addr, addr
                        );
                        found.push(device);
                    }
                    Some(device) => {
                        trace!("ignored duplicate tuya discovery for {}", device.device_id);
                    }
                    None => {
                        trace!("ignored unparsable tuya UDP candidate from {}", addr);
                    }
                }
            }
            Ok(Err(err)) => return Err(err.into()),
            Err(_) => break,
        }
    }

    Ok(found)
}

async fn recv_from_any(
    left: &UdpSocket,
    right: &UdpSocket,
) -> std::io::Result<(Vec<u8>, SocketAddr)> {
    let mut left_buffer = [0; 4096];
    let mut right_buffer = [0; 4096];

    tokio::select! {
        result = left.recv_from(&mut left_buffer) => {
            result.map(|(size, addr)| (left_buffer[..size].to_vec(), addr))
        }
        result = right.recv_from(&mut right_buffer) => {
            result.map(|(size, addr)| (right_buffer[..size].to_vec(), addr))
        }
    }
}

fn parse_discovery_packet(packet: &[u8], addr: SocketAddr) -> Option<DiscoveredDevice> {
    let value = parse_json_packet(packet).or_else(|| parse_framed_packet(packet))?;

    trace!("tuya discovery packet: {value}");

    let device_id = value
        .get("gwId")
        .or_else(|| value.get("devId"))
        .or_else(|| value.get("id"))?
        .as_str()?
        .to_string();

    let control_addr = value
        .get("ip")
        .and_then(Value::as_str)
        .and_then(|ip| ip.parse::<IpAddr>().ok())
        .map(|ip| SocketAddr::new(ip, CONTROL_PORT))
        .unwrap_or_else(|| SocketAddr::new(addr.ip(), CONTROL_PORT));

    Some(DiscoveredDevice {
        device_id,
        addr: control_addr,
    })
}

fn discovery_timeout() -> Duration {
    std::env::var("TUYA_DISCOVERY_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_DISCOVERY_TIMEOUT)
}

fn parse_json_packet(packet: &[u8]) -> Option<Value> {
    let json_start = packet.iter().position(|byte| *byte == b'{')?;
    let json_end = packet.iter().rposition(|byte| *byte == b'}')?;
    serde_json::from_slice(&packet[json_start..=json_end]).ok()
}

fn parse_framed_packet(packet: &[u8]) -> Option<Value> {
    if packet.len() < 24 {
        trace!(
            "tuya framed discovery parse skipped: packet too short ({} bytes)",
            packet.len()
        );
        return None;
    }

    let prefix = u32::from_be_bytes(packet[0..4].try_into().ok()?);
    if prefix != PREFIX_55AA {
        trace!("tuya framed discovery parse skipped: unsupported prefix {prefix:#x}");
        return None;
    }

    let seq = u32::from_be_bytes(packet[4..8].try_into().ok()?);
    let command = u32::from_be_bytes(packet[8..12].try_into().ok()?);
    let length = u32::from_be_bytes(packet[12..16].try_into().ok()?) as usize;
    trace!(
        "tuya 55aa discovery frame: seq={}, command={}, length={}, packet bytes={}",
        seq,
        command,
        length,
        packet.len()
    );
    if length < 8 || packet.len() < 16 + length {
        trace!(
            "tuya framed discovery parse skipped: invalid length {length}, packet bytes {}",
            packet.len()
        );
        return None;
    }

    let payload = &packet[16..16 + length - 8];
    parse_json_packet(payload)
        .or_else(|| decrypt_discovery_payload(payload))
        .or_else(|| payload.get(4..).and_then(decrypt_discovery_payload))
}

fn decrypt_discovery_payload(payload: &[u8]) -> Option<Value> {
    let mut encrypted = payload.to_vec();
    let decrypted = match crypto::ebc::decrypt(&mut encrypted, UDP_KEY) {
        Ok(decrypted) => decrypted,
        Err(_) => {
            trace!(
                "tuya framed discovery decrypt failed for payload length {}",
                payload.len()
            );
            return None;
        }
    };
    parse_json_packet(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport::Room;

    #[test]
    fn test_parse_discovery_packet() {
        let addr = "10.0.1.15:6668".parse().unwrap();
        let device = parse_discovery_packet(br#"xxxx{"gwId":"device-1"}yyyy"#, addr).unwrap();
        assert_eq!(
            device,
            DiscoveredDevice {
                device_id: "device-1".to_string(),
                addr,
            }
        );
    }

    #[test]
    fn test_parse_discovery_packet_uses_payload_ip_with_control_port() {
        let source_addr = "10.0.1.15:54943".parse().unwrap();
        let device =
            parse_discovery_packet(br#"{"gwId":"device-1","ip":"10.0.1.92"}"#, source_addr)
                .unwrap();

        assert_eq!(device.device_id, "device-1");
        assert_eq!(device.addr, "10.0.1.92:6668".parse().unwrap());
    }

    #[test]
    fn test_parse_encrypted_framed_discovery_packet() {
        let mut payload = br#"{"gwId":"device-1"}"#.to_vec();
        let encrypted = crypto::ebc::encrypt(&mut payload, UDP_KEY)
            .unwrap()
            .to_vec();
        let mut packet = Vec::new();
        packet.extend(PREFIX_55AA.to_be_bytes());
        packet.extend(1u32.to_be_bytes());
        packet.extend(19u32.to_be_bytes());
        packet.extend((encrypted.len() as u32 + 8).to_be_bytes());
        packet.extend(encrypted);
        packet.extend(0u32.to_be_bytes());
        packet.extend(0x0000_aa55u32.to_be_bytes());

        let addr = "10.0.1.15:6667".parse().unwrap();
        let device = parse_discovery_packet(&packet, addr).unwrap();
        assert_eq!(device.device_id, "device-1");
    }

    #[test]
    fn test_resolve_exact_match() {
        let addr = "10.0.1.15:6668".parse().unwrap();
        let configs = vec![DeviceConfig {
            room: Room::Bedroom,
            device_id: "device-1".to_string(),
            local_key: "0123456789abcdef".to_string(),
            version: "3.3".to_string(),
            host: None,
        }];
        let discovered = vec![DiscoveredDevice {
            device_id: "device-1".to_string(),
            addr,
        }];

        let resolved = resolve_devices_from_discovered(configs, discovered).unwrap();
        assert_eq!(resolved[0].addr, addr);
    }

    #[test]
    fn test_resolve_fallback_host() {
        let configs = vec![DeviceConfig {
            room: Room::Bedroom,
            device_id: "device-1".to_string(),
            local_key: "0123456789abcdef".to_string(),
            version: "3.3".to_string(),
            host: Some("10.0.1.15".to_string()),
        }];

        let resolved = resolve_devices_from_discovered(configs, vec![]).unwrap();
        assert_eq!(resolved[0].addr, "10.0.1.15:6668".parse().unwrap());
    }

    #[test]
    fn test_resolve_missing() {
        let configs = vec![DeviceConfig {
            room: Room::Bedroom,
            device_id: "device-1".to_string(),
            local_key: "0123456789abcdef".to_string(),
            version: "3.3".to_string(),
            host: None,
        }];

        assert!(matches!(
            resolve_devices_from_discovered(configs, vec![]),
            Err(Error::UnresolvedDevices(_))
        ));
    }
}
