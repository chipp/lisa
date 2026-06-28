use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use transport::elzhbieta::{Action, State};
use transport::Room;

use crate::discovery::{resolve_devices, ResolvedDevice};
use crate::error::{Error, Result};
use crate::profile::{dps_from_action, state_from_dps};
use crate::protocol::{send_json, Command};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DeviceConfig {
    pub room: Room,
    pub device_id: String,
    pub local_key: String,
    pub version: String,
    #[serde(default)]
    pub host: Option<String>,
}

#[derive(Clone, Debug)]
struct Device {
    config: DeviceConfig,
    addr: SocketAddr,
    key: [u8; 16],
}

#[derive(Clone, Debug)]
pub struct Client {
    configs: Vec<DeviceConfig>,
    devices: HashMap<Room, Device>,
}

impl Client {
    pub async fn connect(configs: Vec<DeviceConfig>) -> Result<Self> {
        let mut client = Self {
            configs,
            devices: HashMap::new(),
        };
        client.rediscover().await?;
        Ok(client)
    }

    pub async fn rediscover(&mut self) -> Result<()> {
        let resolved = resolve_devices(self.configs.clone()).await?;
        self.devices = resolved
            .into_iter()
            .map(device_from_resolved)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|device| (device.config.room, device))
            .collect();
        Ok(())
    }

    pub async fn state(&mut self, room: Room) -> Result<State> {
        match self.state_once(room).await {
            Ok(state) => Ok(state),
            Err(err) => {
                debug!("tuya state failed for {room}: {err}; rediscovering");
                self.rediscover().await?;
                self.state_once(room).await
            }
        }
    }

    pub async fn apply(&mut self, action: Action) -> Result<()> {
        match self.apply_once(action.clone()).await {
            Ok(()) => Ok(()),
            Err(err) => {
                debug!(
                    "tuya action failed for {}: {err}; rediscovering",
                    action.room
                );
                self.rediscover().await?;
                self.apply_once(action).await
            }
        }
    }

    pub fn rooms(&self) -> impl Iterator<Item = Room> + '_ {
        self.devices.keys().copied()
    }

    async fn state_once(&self, room: Room) -> Result<State> {
        let device = self.device(room)?;
        let response = send(device, Command::Status, status_payload(device)).await?;
        let dps = response_dps(&response)?;
        state_from_dps(room, dps)
    }

    async fn apply_once(&self, action: Action) -> Result<()> {
        let device = self.device(action.room)?;
        let response = send(device, Command::Control, control_payload(device, action)).await?;
        let _ = response_dps(&response).ok();
        Ok(())
    }

    fn device(&self, room: Room) -> Result<&Device> {
        self.devices
            .get(&room)
            .ok_or_else(|| Error::UnresolvedDevices(vec![room.to_string()]))
    }
}

fn device_from_resolved(resolved: ResolvedDevice) -> Result<Device> {
    if resolved.config.version != "3.3" {
        return Err(Error::UnsupportedVersion(resolved.config.version));
    }

    let key = local_key(&resolved.config.local_key)?;
    info!(
        "resolved tuya AC {} at {} for {}",
        resolved.config.device_id, resolved.addr, resolved.config.room
    );

    Ok(Device {
        config: resolved.config,
        addr: resolved.addr,
        key,
    })
}

async fn send(device: &Device, command: Command, payload: Value) -> Result<Value> {
    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(device.addr)).await??;
    let mut seq = 0;
    timeout(
        Duration::from_secs(5),
        send_json(&mut stream, &mut seq, command, &payload, device.key),
    )
    .await?
}

fn status_payload(device: &Device) -> Value {
    json!({
        "gwId": device.config.device_id,
        "devId": device.config.device_id,
        "uid": device.config.device_id,
        "t": unix_timestamp(),
    })
}

fn control_payload(device: &Device, action: Action) -> Value {
    json!({
        "devId": device.config.device_id,
        "uid": device.config.device_id,
        "t": unix_timestamp(),
        "dps": Value::Object(dps_from_action(action.action_type)),
    })
}

fn response_dps(response: &Value) -> Result<&Map<String, Value>> {
    response
        .get("dps")
        .and_then(Value::as_object)
        .ok_or(Error::MissingDps("dps"))
}

fn local_key(key: &str) -> Result<[u8; 16]> {
    let bytes = key.as_bytes();
    if bytes.len() != 16 {
        return Err(Error::InvalidLocalKey);
    }

    let mut result = [0; 16];
    result.copy_from_slice(bytes);
    Ok(result)
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}
