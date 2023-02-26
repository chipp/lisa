mod error;
use error::Error;

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{DeviceType, Result, StatePayload};

use log::{debug, error, info};
use tokio::time::timeout;

use inspinia::{
    download_template, Device, DeviceManager, FanSpeed, PortName, PortState, PortType,
    ReceivedMessage, RegisterMessage, Room, UpdateMessageContent, UpdateStateMessage, WsClient,
    WsError,
};

pub struct InspiniaClient {
    db_path: PathBuf,
    target_id: String,
    client: Option<WsClient>,
    initial_state: Vec<PortState>,
}

impl InspiniaClient {
    pub async fn new(token: String) -> Result<InspiniaClient> {
        let target_id = token_as_uuid(format!("{:x}", md5::compute(token)));
        let db_path = download_template(&target_id).await?;

        let (client, initial_state) = Self::connect(target_id.clone()).await?;

        info!("initialized");

        Ok(InspiniaClient {
            db_path,
            target_id,
            client: Some(client),
            initial_state,
        })
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        if let Some(old) = self.client.take() {
            old.close().await;
            info!("closed active client");
        } else {
            info!("no active client");
        }

        let (client, port_states) = timeout(
            Duration::from_secs(5),
            Self::connect(self.target_id.clone()),
        )
        .await??;
        info!("reconnected");

        self.client = Some(client);
        self.initial_state = port_states;

        Ok(())
    }

    async fn connect(target_id: String) -> Result<(WsClient, Vec<PortState>)> {
        let mut client =
            WsClient::connect("3af0e0ef-c4dd-4d7e-bb42-f4d24383ed3f", target_id).await?;

        debug!("connected web socket");

        client
            .send_message(RegisterMessage::new("2", "alisa", ""))
            .await?;

        debug!("sent register");

        loop {
            if let Ok(ReceivedMessage {
                code: _,
                message: Some(message),
            }) = client.read_message().await
            {
                let states: Vec<PortState> = serde_json::from_value(message)?;

                return Ok((client, states));
            }
        }
    }
}

impl InspiniaClient {
    pub async fn read(&mut self) -> Result<StatePayload> {
        debug!("read next");
        debug!("initial state {:?}", self.initial_state.len());

        while let Some(state) = self.initial_state.pop() {
            debug!("found initial state {:?} {:?}", state.id, state.value);

            if let Some(update) = Self::parse_initial_state(state, &self.db_path) {
                debug!("prepared update {:?}", update);

                return Ok(update);
            }
        }

        debug!("reading from web socket");

        let client = self.client.as_mut().ok_or(WsError::StreamClosed)?;

        loop {
            match client.read_message().await {
                Ok(payload) => match payload.code.as_str() {
                    "100" => {
                        if let ReceivedMessage {
                            code: _,
                            message: Some(message),
                        } = payload
                        {
                            let update: UpdateMessageContent =
                                serde_json::from_value(message).expect("valid update");

                            if let Ok(update) =
                                Self::state_payload(&update.id, &update.value, &self.db_path)
                            {
                                return Ok(update);
                            }
                        }
                    }
                    "203" => {
                        debug!(
                            "alive: {}",
                            payload
                                .message
                                .unwrap_or(serde_json::Value::Null)
                                .as_str()
                                .unwrap_or_default()
                        )
                    }
                    _ => info!("unsupported message: {:?}", payload),
                },
                Err(WsError::Pong) => (),
                Err(error) => {
                    error!("error reading Inspinia {:?}", error);
                    return Err(error.into());
                }
            }
        }
    }

    fn parse_initial_state(state: PortState, db_path: &Path) -> Option<StatePayload> {
        let value = state.value.as_ref()?;
        Self::state_payload(&state.id, value, db_path).ok()
    }

    fn state_payload(port_id: &str, value: &str, db_path: &Path) -> Result<StatePayload> {
        let devices = Self::get_devices(db_path)?;

        for (device_type, device) in devices {
            if let Some(port) = device.ports.get(port_id) {
                return Ok(StatePayload {
                    device_type,
                    room: device.room,
                    capability: port.name.into(),
                    value: value.to_string(),
                });
            }
        }

        Err(Error::UnsupportedDevice(port_id.to_string()).into())
    }

    fn get_devices(db_path: &Path) -> Result<[(DeviceType, Device); 5]> {
        let device_manager = DeviceManager::new(&db_path)?;

        Ok([
            (
                DeviceType::Recuperator,
                device_manager.get_recuperator_in_room(Room::LivingRoom)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(Room::Bedroom)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(Room::Nursery)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(Room::HomeOffice)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(Room::LivingRoom)?,
            ),
        ])
    }

    fn get_thermostats(db_path: &Path) -> Result<[Device; 4]> {
        let device_manager = DeviceManager::new(&db_path)?;
        Ok([
            device_manager.get_thermostat_in_room(Room::Bedroom)?,
            device_manager.get_thermostat_in_room(Room::Nursery)?,
            device_manager.get_thermostat_in_room(Room::HomeOffice)?,
            device_manager.get_thermostat_in_room(Room::LivingRoom)?,
        ])
    }

    fn get_recuperator(db_path: &Path) -> Result<Device> {
        let device_manager = DeviceManager::new(&db_path)?;
        Ok(device_manager.get_recuperator_in_room(Room::LivingRoom)?)
    }
}

impl InspiniaClient {
    pub async fn set_thermostat_enabled(&mut self, value: bool, room: Room) -> Result<()> {
        info!("toggle thermostat in room {:?} = {}", room, value);

        let value = if value { "1" } else { "0" };

        let thermostats = Self::get_thermostats(&self.db_path)?;
        let client = self.client.as_mut().ok_or(WsError::StreamClosed)?;

        for thermostat in thermostats.iter() {
            if thermostat.room != room {
                continue;
            }

            for (id, port) in thermostat.ports.iter() {
                if let (PortName::OnOff, PortType::Output) = (&port.name, &port.r#type) {
                    client
                        .send_message(UpdateStateMessage::new(false, &id, &port.name, &value))
                        .await?;

                    return Ok(());
                } else {
                    continue;
                }
            }
        }

        panic!("set_is_enabled_in_room")
    }

    pub async fn set_thermostat_temperature(&mut self, value: f64, room: Room) -> Result<()> {
        let temp = value.to_string();

        info!("set temperature in room {:?} = {}", room, temp);

        let thermostats = Self::get_thermostats(&self.db_path)?;
        let client = self.client.as_mut().ok_or(WsError::StreamClosed)?;

        for thermostat in thermostats.iter() {
            if thermostat.room != room {
                continue;
            }

            for (id, port) in thermostat.ports.iter() {
                if let (PortName::SetTemp, PortType::Output) = (&port.name, &port.r#type) {
                    client
                        .send_message(UpdateStateMessage::new(false, &id, &port.name, &temp))
                        .await?;

                    return Ok(());
                } else {
                    continue;
                }
            }
        }

        panic!("set_temperature_in_room")
    }
}

impl InspiniaClient {
    pub async fn set_recuperator_enabled(&mut self, value: bool) -> Result<()> {
        info!("toggle recuperator = {}", value);

        let value = if value { "1" } else { "0" };

        let recuperator = Self::get_recuperator(&self.db_path)?;
        let client = self.client.as_mut().ok_or(WsError::StreamClosed)?;

        for (id, port) in recuperator.ports.iter() {
            if let (PortName::OnOff, PortType::Output) = (&port.name, &port.r#type) {
                client
                    .send_message(UpdateStateMessage::new(false, &id, &port.name, &value))
                    .await?;

                return Ok(());
            } else {
                continue;
            }
        }

        panic!("set_is_enabled_on_recuperator")
    }

    pub async fn set_recuperator_fan_speed(&mut self, value: FanSpeed) -> Result<()> {
        info!("change fan speed on recuperator = {:?}", value);

        let value = value.to_string();

        let device_manager = DeviceManager::new(&self.db_path)?;
        let recuperator = device_manager.get_recuperator_in_room(Room::LivingRoom)?;
        let client = self.client.as_mut().ok_or(WsError::StreamClosed)?;

        for (id, port) in recuperator.ports.iter() {
            if let (PortName::FanSpeed, PortType::Output) = (&port.name, &port.r#type) {
                client
                    .send_message(UpdateStateMessage::new(false, &id, &port.name, &value))
                    .await?;

                return Ok(());
            } else {
                continue;
            }
        }

        panic!("set_fan_speed_on_recuperator")
    }
}

fn token_as_uuid(mut token: String) -> String {
    assert!(token.len() == 32);

    token.insert(20, '-');
    token.insert(16, '-');
    token.insert(12, '-');
    token.insert(8, '-');

    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_as_uuid() {
        assert_eq!(
            token_as_uuid("4cfdc2e157eefe6facb983b1d557b3a1".to_string()),
            "4cfdc2e1-57ee-fe6f-acb9-83b1d557b3a1".to_string()
        );
    }
}
