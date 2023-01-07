use std::net::SocketAddr;
use std::sync::Arc;

use crate::{Result, SocketHandler, StateManager};
use elisheba::{CommandResponse as VacuumCommandResponse, Packet, SensorData, SensorRoom};
use log::info;

use tokio::sync::{mpsc, Mutex};

pub async fn read_from_socket(
    socket_handler: SocketHandler,
    addr: SocketAddr,
    cmd_res_tx: mpsc::Sender<VacuumCommandResponse>,
    state_manager: Arc<Mutex<StateManager>>,
) -> Result<()> {
    info!("A client did connect {}", addr);

    let mut socket_handler = socket_handler;
    let _ = socket_handler
        .read_packets(|packet| {
            let cmd_res_tx = cmd_res_tx.clone();
            let state_manager = state_manager.clone();

            async move {
                match packet {
                    Packet::CommandResponse(response) => cmd_res_tx.send(response).await.unwrap(),
                    Packet::VacuumStatus(status) => {
                        let mut state = state_manager.clone().lock_owned().await;

                        state.vacuum_state.set_battery(status.battery);
                        state.vacuum_state.set_is_enabled(status.is_enabled);
                        state.vacuum_state.set_work_speed(status.work_speed);
                    }
                    Packet::SensorData(sensor_data) => {
                        let mut state = state_manager.clone().lock_owned().await;

                        let room_state = match sensor_data.room() {
                            SensorRoom::Bedroom => &mut state.bedroom_sensor_state,
                            SensorRoom::HomeOffice => &mut state.home_office_sensor_state,
                            SensorRoom::Kitchen => &mut state.kitchen_sensor_state,
                        };

                        match sensor_data {
                            SensorData::Temperature {
                                room: _,
                                temperature,
                            } => {
                                room_state.set_temperature(temperature);
                            }
                            SensorData::Humidity { room: _, humidity } => {
                                room_state.set_humidity(humidity);
                            }
                            SensorData::Battery { room: _, battery } => {
                                room_state.set_battery(battery);
                            }
                            SensorData::TemperatureAndHumidity {
                                room: _,
                                temperature,
                                humidity,
                            } => {
                                room_state.set_temperature(temperature);
                                room_state.set_humidity(humidity);
                            }
                        }
                    }
                }
            }
        })
        .await;

    info!("The client did disconnect {}", addr);

    Ok(())
}
