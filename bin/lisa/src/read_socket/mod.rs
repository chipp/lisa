mod handler;
pub use handler::Handler;

use std::net::SocketAddr;
use std::sync::Arc;

use crate::{Result, Room, StateManager};
use elisheba::{CommandResponse as VacuumCommandResponse, Packet, SensorData, SensorRoom};
use log::info;

use tokio::sync::{mpsc, Mutex};

pub async fn read_from_socket(
    handler: Handler,
    addr: SocketAddr,
    cmd_res_tx: mpsc::Sender<VacuumCommandResponse>,
    state_manager: Arc<Mutex<StateManager>>,
) -> Result<()> {
    info!("A client did connect {}", addr);

    let mut handler = handler;
    let _ = handler
        .read_packets(|packet| {
            let cmd_res_tx = cmd_res_tx.clone();
            let state_manager = state_manager.clone();

            async move {
                match packet {
                    Packet::CommandResponse(response) => cmd_res_tx.send(response).await.unwrap(),
                    Packet::VacuumStatus(status) => {
                        let mut state = state_manager.clone().lock_owned().await;
                        let vacuum_state = state.vacuum_state();

                        vacuum_state.set_battery(status.battery);
                        vacuum_state.set_is_enabled(status.is_enabled);
                        vacuum_state.set_work_speed(status.work_speed);
                    }
                    Packet::SensorData(sensor_data) => {
                        let mut state = state_manager.clone().lock_owned().await;

                        if let Some(room_state) =
                            state.sensor_state_in_room(map_room(sensor_data.room()))
                        {
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
            }
        })
        .await;

    info!("The client did disconnect {}", addr);

    Ok(())
}

fn map_room(room: &SensorRoom) -> Room {
    match room {
        SensorRoom::Bedroom => Room::Bedroom,
        SensorRoom::HomeOffice => Room::HomeOffice,
        SensorRoom::Kitchen => Room::Kitchen,
        SensorRoom::Nursery => Room::Nursery,
    }
}
