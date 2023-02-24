mod error;
use error::Error;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::{DeviceType, Result, StateManager};

use log::{debug, error, info};
use tokio::sync::Mutex;
use tokio::task::{self, JoinHandle};
use tokio::time;

use alisa::{
    download_template, Device, DeviceManager, FanSpeed, KeepAliveMessage, PortName, PortState,
    PortType, RegisterMessage, Room, UpdateMessageContent, UpdateStateMessage, WSClient,
};

#[derive(Clone)]
pub struct InspiniaController {
    db_path: PathBuf,
    client: WSClient,
    state_manager: Arc<Mutex<StateManager>>,
    keep_alive_handle: Arc<JoinHandle<()>>,
}

impl InspiniaController {
    pub async fn new(
        token: String,
        state_manager: Arc<Mutex<StateManager>>,
    ) -> Result<InspiniaController> {
        let target_id = token_as_uuid(format!("{:x}", md5::compute(token)));

        let db_path = download_template(&target_id).await?;

        let (client, port_states) = Self::connect(target_id).await?;
        Self::set_current_state(port_states, state_manager.clone(), &db_path).await?;

        let keep_alive_handle = Self::keep_alive(client.clone());

        Ok(InspiniaController {
            db_path,
            client,
            state_manager,
            keep_alive_handle: Arc::from(keep_alive_handle),
        })
    }

    async fn set_current_state(
        port_states: Vec<PortState>,
        state_manager: Arc<Mutex<StateManager>>,
        db_path: &Path,
    ) -> Result<()> {
        struct PortContext {
            device_type: DeviceType,
            room: Room,
            port_name: PortName,
        }

        let device_manager = DeviceManager::new(&db_path)?;
        let devices = [
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
            (
                DeviceType::Recuperator,
                device_manager.get_recuperator_in_room(Room::LivingRoom)?,
            ),
        ];

        let mut ports: HashMap<String, PortContext> = HashMap::new();

        for (device_type, device) in devices {
            for (key, port) in device.ports {
                ports.insert(
                    key,
                    PortContext {
                        device_type,
                        room: device.room,
                        port_name: port.name,
                    },
                );
            }
        }

        for port_state in port_states {
            let value = if let Some(value) = port_state.value {
                value
            } else {
                continue;
            };

            if let Some(context) = ports.get(&port_state.id) {
                match context.device_type {
                    DeviceType::Recuperator => {
                        let mut state_manager = state_manager.clone().lock_owned().await;
                        let state = state_manager.recuperator_state();

                        match context.port_name {
                            PortName::OnOff => state.set_is_enabled(value == "1"),
                            PortName::FanSpeed => {
                                if let Ok(value) = FanSpeed::try_from(value.as_str()) {
                                    state.set_fan_speed(value);
                                }
                            }
                            PortName::SetTemp | PortName::RoomTemp | PortName::Mode => (),
                        }
                    }
                    DeviceType::Thermostat => {
                        let mut state_manager = state_manager.clone().lock_owned().await;
                        let state = if let Some(state) =
                            state_manager.thermostat_state_in_room(map_room(&context.room))
                        {
                            state
                        } else {
                            continue;
                        };

                        match context.port_name {
                            PortName::OnOff => state.set_is_enabled(value == "1"),
                            PortName::SetTemp => {
                                if let Ok(value) = f32::from_str(&value) {
                                    state.set_target_temperature(value);
                                }
                            }
                            PortName::RoomTemp => {
                                if let Ok(value) = f32::from_str(&value) {
                                    state.set_room_temperature(value);
                                }
                            }
                            PortName::Mode | PortName::FanSpeed => (),
                        }
                    }
                    DeviceType::TemperatureSensor | DeviceType::VacuumCleaner => continue,
                }
            }
        }

        Ok(())
    }

    async fn connect(target_id: String) -> Result<(WSClient, Vec<PortState>)> {
        let mut client =
            WSClient::connect("3af0e0ef-c4dd-4d7e-bb42-f4d24383ed3f", target_id).await?;

        client
            .send_message(RegisterMessage::new("2", "alisa", ""))
            .await?;

        loop {
            if let Some(payload) = client.read_message().await {
                let states: Vec<PortState> = serde_json::from_value(payload.message)?;

                return Ok((client, states));
            }
        }
    }

    fn keep_alive(mut client: WSClient) -> JoinHandle<()> {
        task::spawn(async move {
            let mut timer = time::interval(Duration::from_secs(1));

            loop {
                timer.tick().await;
                match client.send_message(KeepAliveMessage::new()).await {
                    Ok(()) => debug!("keep alive"),
                    Err(err) => error!("keep alive failed {}", err),
                }
            }
        })
    }
}

impl Drop for InspiniaController {
    fn drop(&mut self) {
        self.keep_alive_handle.abort();
    }
}

impl InspiniaController {
    pub async fn listen(&mut self) -> Result<()> {
        loop {
            if let Some(payload) = self.client.read_message().await {
                match payload.code.as_str() {
                    "100" => {
                        let update: UpdateMessageContent =
                            serde_json::from_value(payload.message).expect("valid update");

                        match self.update_state(&update.id, &update.value).await {
                            Ok(()) => (),
                            Err(_err) => (),
                            // error!("unable to update device state {} {:#?}", err, update)
                        }
                    }
                    "203" => {
                        debug!("alive: {}", payload.message.as_str().unwrap_or_default())
                    }
                    _ => info!("unsupported message: {:?}", payload),
                }
            }
        }
    }

    fn get_thermostats(&self) -> Result<[Device; 4]> {
        let device_manager = DeviceManager::new(&self.db_path)?;
        Ok([
            device_manager.get_thermostat_in_room(Room::Bedroom)?,
            device_manager.get_thermostat_in_room(Room::Nursery)?,
            device_manager.get_thermostat_in_room(Room::HomeOffice)?,
            device_manager.get_thermostat_in_room(Room::LivingRoom)?,
        ])
    }

    fn get_recuperator(&self) -> Result<Device> {
        let device_manager = DeviceManager::new(&self.db_path)?;
        let recuperator = device_manager.get_recuperator_in_room(Room::LivingRoom)?;
        Ok(recuperator)
    }

    async fn update_state(&self, port_id: &str, value: &str) -> Result<()> {
        if self.update_recuperator(port_id, value).await? {
            return Ok(());
        }

        let thermostats = self.get_thermostats()?;

        for thermostat in thermostats.iter() {
            let mut state_manager = self.state_manager.clone().lock_owned().await;

            if let Some(port) = thermostat.ports.get(port_id) {
                if let Some(state) =
                    state_manager.thermostat_state_in_room(map_room(&thermostat.room))
                {
                    match port.name {
                        PortName::OnOff => state.set_is_enabled(value == "1"),
                        PortName::SetTemp => {
                            if let Ok(value) = f32::from_str(&value) {
                                state.set_target_temperature(value);
                            }
                        }
                        PortName::RoomTemp => {
                            if let Ok(value) = f32::from_str(&value) {
                                state.set_room_temperature(value);
                            }
                        }
                        PortName::Mode | PortName::FanSpeed => (),
                    }
                }

                return Ok(());
            }
        }

        Err(Error::UnsupportedDevice(port_id.to_string()).into())
    }

    async fn update_recuperator(&self, port_id: &str, value: &str) -> Result<bool> {
        let recuperator = self.get_recuperator()?;
        let mut state_manager = self.state_manager.clone().lock_owned().await;

        if let Some(port) = recuperator.ports.get(port_id) {
            let state = state_manager.recuperator_state();

            match port.name {
                PortName::OnOff => state.set_is_enabled(value == "1"),
                PortName::FanSpeed => {
                    if let Ok(value) = FanSpeed::try_from(value) {
                        state.set_fan_speed(value);
                    }
                }
                PortName::Mode | PortName::RoomTemp | PortName::SetTemp => (),
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl InspiniaController {
    pub async fn set_is_enabled_in_room(&mut self, value: bool, room: Room) -> Result<()> {
        info!("toggle thermostat in room {:?} = {}", room, value);

        let value = if value { "1" } else { "0" };

        let thermostats = self.get_thermostats()?;

        for thermostat in thermostats.iter() {
            if thermostat.room != room {
                continue;
            }

            for (id, port) in thermostat.ports.iter() {
                if let (PortName::OnOff, PortType::Output) = (&port.name, &port.r#type) {
                    self.client
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

    pub async fn set_temperature_in_room(
        &mut self,
        value: f32,
        relative: bool,
        room: Room,
    ) -> Result<()> {
        let temp = if relative {
            let mut state_manager = self.state_manager.clone().lock_owned().await;
            if let Some(state) = state_manager.thermostat_state_in_room(map_room(&room)) {
                state.target_temperature() + value
            } else {
                0.0
            }
        } else {
            value
        }
        .to_string();

        info!("set temperature in room {:?} = {}", room, temp);

        let thermostats = self.get_thermostats()?;

        for thermostat in thermostats.iter() {
            if thermostat.room != room {
                continue;
            }

            for (id, port) in thermostat.ports.iter() {
                if let (PortName::SetTemp, PortType::Output) = (&port.name, &port.r#type) {
                    self.client
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

impl InspiniaController {
    pub async fn set_is_enabled_on_recuperator(&mut self, value: bool) -> Result<()> {
        info!("toggle recuperator = {}", value);

        let value = if value { "1" } else { "0" };

        let recuperator = self.get_recuperator()?;

        for (id, port) in recuperator.ports.iter() {
            if let (PortName::OnOff, PortType::Output) = (&port.name, &port.r#type) {
                self.client
                    .send_message(UpdateStateMessage::new(false, &id, &port.name, &value))
                    .await?;

                return Ok(());
            } else {
                continue;
            }
        }

        panic!("set_is_enabled_on_recuperator")
    }

    pub async fn set_fan_speed_on_recuperator(&mut self, value: FanSpeed) -> Result<()> {
        info!("change fan speed on recuperator = {:?}", value);

        let value = value.to_string();

        let device_manager = DeviceManager::new(&self.db_path)?;
        let recuperator = device_manager.get_recuperator_in_room(Room::LivingRoom)?;

        for (id, port) in recuperator.ports.iter() {
            if let (PortName::FanSpeed, PortType::Output) = (&port.name, &port.r#type) {
                self.client
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

fn map_room(room: &Room) -> crate::Room {
    match room {
        Room::Bedroom => crate::Room::Bedroom,
        Room::HomeOffice => crate::Room::HomeOffice,
        Room::LivingRoom => crate::Room::LivingRoom,
        Room::Nursery => crate::Room::Nursery,
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

    #[test]
    fn test_map_room() {
        assert_eq!(map_room(&Room::Bedroom), crate::Room::Bedroom);
        assert_eq!(map_room(&Room::HomeOffice), crate::Room::HomeOffice);
        assert_eq!(map_room(&Room::LivingRoom), crate::Room::LivingRoom);
        assert_eq!(map_room(&Room::Nursery), crate::Room::Nursery);
    }
}
