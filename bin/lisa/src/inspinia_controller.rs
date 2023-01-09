mod error;
use error::Error;

use std::sync::Arc;
use std::{path::PathBuf, str::FromStr};

use crate::{Result, StateManager};

use log::info;
use tokio::sync::Mutex;

use alisa::{
    download_template, Device, DeviceManager, FanSpeed, PortName, PortState, PortType,
    RegisterMessage, Room, UpdateMessageContent, UpdateStateMessage, WSClient,
};

#[derive(Clone)]
pub struct InspiniaController {
    db_path: PathBuf,
    client: WSClient,
    state_manager: Arc<Mutex<StateManager>>,
}

impl InspiniaController {
    pub async fn new(
        token: String,
        state_manager: Arc<Mutex<StateManager>>,
    ) -> Result<InspiniaController> {
        let target_id = token_as_uuid(format!("{:x}", md5::compute(token)));

        let db_path = download_template(&target_id).await?;
        let (client, _port_states) = Self::connect(target_id).await?;

        Ok(InspiniaController {
            db_path,
            client,
            state_manager,
        })
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
                    _ => println!("unsupported message: {:?}", payload),
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

    async fn update_state(&self, port_id: &str, value: &str) -> Result<()> {
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

        let device_manager = DeviceManager::new(&self.db_path)?;
        let recuperator = device_manager.get_recuperator_in_room(Room::LivingRoom)?;

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
