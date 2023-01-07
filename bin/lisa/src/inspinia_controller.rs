use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use crate::{Result, StateManager, ThermostatState};

use log::info;
use rusqlite::{params, CachedStatement, Connection};
use tokio::sync::Mutex;

use alisa::{
    download_template, PortName, PortState, RegisterMessage, UpdateMessageContent, WSClient,
};

pub struct InspiniaController {
    client: WSClient,
    db_path: PathBuf,
}

struct DbManager {
    connection: Connection,
}

impl DbManager {
    fn new<P>(db_path: P) -> Result<DbManager>
    where
        P: AsRef<Path>,
    {
        Ok(DbManager {
            connection: Connection::open(db_path.as_ref())?,
        })
    }

    fn port_select(&self) -> CachedStatement<'_> {
        self.connection
            .prepare_cached(
                "SELECT control_id, value FROM tb_ports
                INNER JOIN tb_port_property ON tb_ports.id = tb_port_property.port_id
                WHERE tb_ports.id = ? AND tb_port_property.name = 'name'",
            )
            .expect("valid port select")
    }
}

impl InspiniaController {
    pub async fn new(token: String) -> Result<InspiniaController> {
        let target_id = token_as_uuid(format!("{:x}", md5::compute(token)));

        let db_path = download_template(&target_id).await?;
        let (client, _port_states) = Self::connect(target_id).await?;

        Ok(InspiniaController { client, db_path })
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

const _LIVING_ROOM_RECUPERATOR: &str = "51602c13-1ada-4712-a883-543549446898";
const LIVING_ROOM_THERMOSTAT: &str = "4b76bfe5-8e69-4afc-ae7f-f6e0070c2cf1";
const BEDROOM_THERMOSTAT: &str = "14a87794-22ce-4a68-a3dd-0d79b43dfb27";
const NURSERY_THERMOSTAT: &str = "d0872196-2cdf-4219-af5f-6d2dd461b18c";
const HOME_OFFICE_THERMOSTAT: &str = "f694e705-8724-4493-99ae-88f02043d7aa";

impl InspiniaController {
    pub async fn listen(&mut self, state_manager: Arc<Mutex<StateManager>>) -> Result<()> {
        loop {
            if let Some(payload) = self.client.read_message().await {
                match payload.code.as_str() {
                    "100" => {
                        let db_manager = DbManager::new(&self.db_path).expect("sqlite connection");

                        let update: UpdateMessageContent =
                            serde_json::from_value(payload.message).expect("valid update");

                        let port_info = db_manager
                            .port_select()
                            .query_row(params![update.id], |r| {
                                Ok((r.get::<_, String>(0)?, r.get::<_, PortName>(1)?))
                            })
                            .unwrap();

                        self.update_state(
                            port_info.0,
                            port_info.1,
                            update.value,
                            state_manager.clone(),
                        )
                        .await;
                    }
                    _ => println!("unsupported message: {:?}", payload),
                }
            }
        }
    }

    async fn update_state(
        &self,
        control_id: String,
        port_name: PortName,
        value: String,
        state_manager: Arc<Mutex<StateManager>>,
    ) {
        match control_id.as_str() {
            BEDROOM_THERMOSTAT => {
                let mut state_manager = state_manager.lock_owned().await;
                self.update_thermostat(
                    &mut state_manager.bedroom_thermostat_state,
                    port_name,
                    value,
                )
            }
            NURSERY_THERMOSTAT => {
                let mut state_manager = state_manager.lock_owned().await;
                self.update_thermostat(
                    &mut state_manager.nursery_thermostat_state,
                    port_name,
                    value,
                )
            }
            HOME_OFFICE_THERMOSTAT => {
                let mut state_manager = state_manager.lock_owned().await;
                self.update_thermostat(
                    &mut state_manager.home_office_thermostat_state,
                    port_name,
                    value,
                )
            }
            LIVING_ROOM_THERMOSTAT => {
                let mut state_manager = state_manager.lock_owned().await;
                self.update_thermostat(
                    &mut state_manager.living_room_thermostat_state,
                    port_name,
                    value,
                )
            }
            _ => (),
        }
    }

    fn update_thermostat(&self, state: &mut ThermostatState, port_name: PortName, value: String) {
        info!("update {:?} {}", port_name, value);

        match port_name {
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
            PortName::Mode => panic!("unsupported"),
            PortName::FanSpeed => panic!("unsupported"),
        }
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
