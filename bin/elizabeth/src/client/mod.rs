mod error;
use error::Error;

mod storage;
use storage::Storage;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, trace};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::Result;
use inspinia::{
    download_template, Device, DeviceManager, PortName, PortState, PortType, ReceivedMessage,
    RegisterMessage, UpdateMessageContent, UpdateStateMessage, WsClient, WsError,
};
use transport::elizabeth::{self, Capability, State};
use transport::DeviceType;

#[derive(Clone)]
pub struct Client {
    db_path: PathBuf,
    client_id: String,
    target_id: String,
    client: WsClient,
    initial_state: Vec<PortState>,
    storage: Arc<Mutex<Storage>>,
    logs_path: PathBuf,
}

impl Client {
    pub async fn new(client_id: String, token: String, logs_path: PathBuf) -> Result<Client> {
        let target_id = token_as_uuid(format!("{:x}", md5::compute(token)));
        let db_path = download_template(&target_id).await?;

        let (client, initial_state) =
            Self::connect(client_id.clone(), target_id.clone(), logs_path.clone()).await?;

        info!("initialized");

        let storage = Storage::new();

        for state in &initial_state {
            if let Some(state) = Self::parse_initial_state(state, &db_path) {
                storage.apply_state(&state).await;
            }
        }

        let storage = Arc::new(Mutex::new(storage));

        Ok(Client {
            db_path,
            client_id,
            target_id,
            client,
            initial_state,
            storage,
            logs_path,
        })
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        let (client, port_states) = timeout(
            Duration::from_secs(5),
            Self::connect(
                self.client_id.clone(),
                self.target_id.clone(),
                self.logs_path.clone(),
            ),
        )
        .await??;
        info!("reconnected");

        self.client = client;
        self.initial_state = port_states;

        Ok(())
    }

    async fn connect(
        client_id: String,
        target_id: String,
        logs_path: PathBuf,
    ) -> Result<(WsClient, Vec<PortState>)> {
        let mut client = WsClient::connect(client_id, target_id, logs_path).await?;

        debug!("connected web socket");

        client
            .send_message(RegisterMessage::new("2", "elizabeth", ""))
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

impl Client {
    pub async fn read(&mut self) -> Result<State> {
        trace!("read next");
        trace!("initial state {:?}", self.initial_state.len());

        while let Some(state) = self.initial_state.pop() {
            trace!("found initial state {:?} {:?}", state.id, state.value);

            if let Some(update) = Self::parse_initial_state(&state, &self.db_path) {
                trace!("prepared update {:?}", update);

                return Ok(update);
            }
        }

        trace!("reading from web socket");

        loop {
            match self.client.read_message().await {
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
                                let storage = self.storage.lock().await;
                                storage.apply_state(&update).await;
                                return Ok(update);
                            }
                        }
                    }
                    "203" => {
                        trace!(
                            "alive: {}",
                            payload
                                .message
                                .unwrap_or(serde_json::Value::Null)
                                .as_str()
                                .unwrap_or_default()
                        )
                    }
                    "404" => {
                        return Err(WsError::StreamClosed.into());
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

    fn parse_initial_state(state: &PortState, db_path: &Path) -> Option<State> {
        let value = state.value.as_ref()?;
        Self::state_payload(&state.id, value, db_path).ok()
    }

    fn state_payload(port_id: &str, value: &str, db_path: &Path) -> Result<State> {
        let devices = Self::get_devices(db_path)?;

        for (device_type, device) in devices {
            if let Some(port) = device.ports.get(port_id) {
                if let Some(capability) = prepare_capability(&port.name, value) {
                    return Ok(State {
                        device_type,
                        room: from_inspinia_room(device.room),
                        capability,
                    });
                }
            }
        }

        Err(Error::UnsupportedDevice(port_id.to_string()).into())
    }

    fn get_devices(db_path: &Path) -> Result<[(DeviceType, Device); 5]> {
        let device_manager = DeviceManager::new(db_path)?;

        Ok([
            (
                DeviceType::Recuperator,
                device_manager.get_recuperator_in_room(inspinia::Room::LivingRoom)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(inspinia::Room::Bedroom)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(inspinia::Room::Nursery)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(inspinia::Room::HomeOffice)?,
            ),
            (
                DeviceType::Thermostat,
                device_manager.get_thermostat_in_room(inspinia::Room::LivingRoom)?,
            ),
        ])
    }

    fn get_thermostats(db_path: &Path) -> Result<[Device; 4]> {
        let device_manager = DeviceManager::new(db_path)?;
        Ok([
            device_manager.get_thermostat_in_room(inspinia::Room::Bedroom)?,
            device_manager.get_thermostat_in_room(inspinia::Room::Nursery)?,
            device_manager.get_thermostat_in_room(inspinia::Room::HomeOffice)?,
            device_manager.get_thermostat_in_room(inspinia::Room::LivingRoom)?,
        ])
    }

    fn get_recuperator(db_path: &Path) -> Result<Device> {
        let device_manager = DeviceManager::new(db_path)?;
        device_manager.get_recuperator_in_room(inspinia::Room::LivingRoom)
    }
}

impl Client {
    pub async fn get_current_state(
        &self,
        room: transport::Room,
        device_type: DeviceType,
    ) -> Vec<Capability> {
        let storage = self.storage.lock().await;

        storage.get_capabilities(room, device_type).await
    }
}

impl Client {
    pub async fn get_thermostat_temperature_in_room(&self, room: transport::Room) -> Result<f32> {
        let storage = self.storage.lock().await;

        let capabilities = storage.get_capabilities(room, DeviceType::Thermostat).await;

        for capability in capabilities {
            if let Capability::Temperature(value) = capability {
                return Ok(value);
            }
        }

        Err(Error::MissingCapability("Temperature", DeviceType::Thermostat, room).into())
    }

    pub async fn set_thermostat_enabled(
        &mut self,
        value: bool,
        room: transport::Room,
    ) -> Result<()> {
        info!("toggle thermostat in room {:?} = {}", room, value);

        let value = if value { "1" } else { "0" };

        let thermostats = Self::get_thermostats(&self.db_path)?;

        for thermostat in thermostats.iter() {
            if from_inspinia_room(thermostat.room) != room {
                continue;
            }

            for (id, port) in thermostat.ports.iter() {
                if let (PortName::OnOff, PortType::Output) = (&port.name, &port.r#type) {
                    self.client
                        .send_message(UpdateStateMessage::new(false, id, &port.name, value))
                        .await?;

                    return Ok(());
                } else {
                    continue;
                }
            }
        }

        panic!("set_is_enabled_in_room")
    }

    pub async fn set_thermostat_temperature(
        &mut self,
        value: f32,
        room: transport::Room,
    ) -> Result<()> {
        let temp = value.to_string();

        info!("set temperature in room {:?} = {}", room, temp);

        let thermostats = Self::get_thermostats(&self.db_path)?;

        for thermostat in thermostats.iter() {
            if from_inspinia_room(thermostat.room) != room {
                continue;
            }

            for (id, port) in thermostat.ports.iter() {
                if let (PortName::SetTemp, PortType::Output) = (&port.name, &port.r#type) {
                    self.client
                        .send_message(UpdateStateMessage::new(false, id, &port.name, &temp))
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

impl Client {
    pub async fn set_recuperator_enabled(&mut self, value: bool) -> Result<()> {
        info!("toggle recuperator = {}", value);

        let value = if value { "1" } else { "0" };

        let recuperator = Self::get_recuperator(&self.db_path)?;

        for (id, port) in recuperator.ports.iter() {
            if let (PortName::OnOff, PortType::Output) = (&port.name, &port.r#type) {
                self.client
                    .send_message(UpdateStateMessage::new(false, id, &port.name, value))
                    .await?;

                return Ok(());
            } else {
                continue;
            }
        }

        panic!("set_is_enabled_on_recuperator")
    }

    pub async fn set_recuperator_fan_speed(&mut self, value: elizabeth::FanSpeed) -> Result<()> {
        info!("change fan speed on recuperator = {:?}", value);

        let value = from_elizabeth_speed(value).to_string();

        let device_manager = DeviceManager::new(&self.db_path)?;
        let recuperator = device_manager.get_recuperator_in_room(inspinia::Room::LivingRoom)?;

        for (id, port) in recuperator.ports.iter() {
            if let (PortName::FanSpeed, PortType::Output) = (&port.name, &port.r#type) {
                self.client
                    .send_message(UpdateStateMessage::new(false, id, &port.name, &value))
                    .await?;

                return Ok(());
            } else {
                continue;
            }
        }

        panic!("set_fan_speed_on_recuperator")
    }
}

fn prepare_capability(name: &PortName, value: &str) -> Option<Capability> {
    match name {
        PortName::OnOff => Some(Capability::IsEnabled(value == "1")),
        PortName::FanSpeed => {
            let fan_speed = inspinia::FanSpeed::from_str(value).ok()?;
            Some(Capability::FanSpeed(from_inspinia_speed(fan_speed)))
        }
        PortName::SetTemp => {
            let value = f32::from_str(value).ok()?;
            Some(Capability::Temperature(value))
        }
        PortName::RoomTemp => {
            let value = f32::from_str(value).ok()?;
            Some(Capability::CurrentTemperature(value))
        }
        PortName::Mode => None,
    }
}

fn from_inspinia_room(room: inspinia::Room) -> transport::Room {
    match room {
        inspinia::Room::Bedroom => transport::Room::Bedroom,
        inspinia::Room::Nursery => transport::Room::Nursery,
        inspinia::Room::HomeOffice => transport::Room::HomeOffice,
        inspinia::Room::LivingRoom => transport::Room::LivingRoom,
    }
}

fn from_inspinia_speed(fan_speed: inspinia::FanSpeed) -> elizabeth::FanSpeed {
    match fan_speed {
        inspinia::FanSpeed::Low => elizabeth::FanSpeed::Low,
        inspinia::FanSpeed::Medium => elizabeth::FanSpeed::Medium,
        inspinia::FanSpeed::High => elizabeth::FanSpeed::High,
    }
}

fn from_elizabeth_speed(speed: elizabeth::FanSpeed) -> inspinia::FanSpeed {
    match speed {
        elizabeth::FanSpeed::Low => inspinia::FanSpeed::Low,
        elizabeth::FanSpeed::Medium => inspinia::FanSpeed::Medium,
        elizabeth::FanSpeed::High => inspinia::FanSpeed::High,
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
