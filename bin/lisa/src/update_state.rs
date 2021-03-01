use std::{str::FromStr, sync::Arc};

use crate::DeviceId;
use crate::DeviceType::*;

use log::info;
use tokio::sync::Mutex;

use alice::{
    ModeFunction, StateCapability, StateUpdateResult, UpdateStateCapability, UpdateStateDevice,
    UpdateStateErrorCode, UpdatedDeviceState,
};
use elisheba::Command;

pub async fn update_devices_state<'a>(
    devices: Vec<UpdateStateDevice<'a>>,
    cmd: Arc<Mutex<crate::SocketHandler>>,
) -> Vec<UpdatedDeviceState> {
    let mut rooms = vec![];
    let mut state = None;
    let mut work_speed = None;

    for device in devices.into_iter() {
        if let Ok(DeviceId { room, device_type }) = DeviceId::from_str(device.id) {
            if device_type != VacuumCleaner {
                // TODO: ??
                continue;
            }

            rooms.push(room);

            if state != None && work_speed != None {
                continue;
            }

            for capability in device.capabilities.into_iter() {
                match capability {
                    StateCapability::OnOff { value } => state = Some(value),
                    StateCapability::Mode {
                        function: ModeFunction::WorkSpeed,
                        mode,
                    } => work_speed = Some(mode),
                }
            }
        }
    }

    info!("update state rooms {:?}", rooms);
    info!("state {:?}", state);
    info!("work_speed {:?}", work_speed);

    if rooms.is_empty() {
        return vec![];
    }

    let set_mode_result;
    let toggle_state_result;

    {
        let mut cmd = cmd.lock_owned().await;
        set_mode_result = match work_speed {
            Some(mode) => Some(
                cmd.send_command(Command::SetWorkSpeed {
                    mode: mode.to_string(),
                })
                .await,
            ),
            None => None,
        };

        toggle_state_result = match state {
            Some(true) => {
                let room_ids = rooms.iter().map(crate::Room::id).collect();

                Some(cmd.send_command(Command::Start { rooms: room_ids }).await)
            }
            Some(false) => {
                let stop = cmd.send_command(Command::Stop).await;
                let home = cmd.send_command(Command::GoHome).await;

                Some(stop.and(home))
            }
            None => None,
        };
    }

    let mut devices = vec![];

    for room in rooms {
        let capabilities;

        match (&toggle_state_result, &set_mode_result) {
            (Some(toggle_state_result), Some(set_mode_result)) => {
                capabilities = vec![
                    UpdateStateCapability::on_off(prepare_result(&toggle_state_result)),
                    UpdateStateCapability::mode(
                        ModeFunction::WorkSpeed,
                        prepare_result(&set_mode_result),
                    ),
                ];
            }
            (Some(toggle_state_result), None) => {
                capabilities = vec![UpdateStateCapability::on_off(prepare_result(
                    &toggle_state_result,
                ))];
            }
            (None, Some(set_mode_result)) => {
                capabilities = vec![UpdateStateCapability::mode(
                    ModeFunction::WorkSpeed,
                    prepare_result(&set_mode_result),
                )];
            }
            _ => return vec![],
        }

        let device_id = DeviceId {
            room,
            device_type: VacuumCleaner,
        };

        devices.push(UpdatedDeviceState::new(device_id.to_string(), capabilities));
    }

    devices
}

fn prepare_result(result: &crate::Result<()>) -> StateUpdateResult {
    match result {
        Ok(_) => StateUpdateResult::ok(),
        Err(_) => {
            StateUpdateResult::error(UpdateStateErrorCode::DeviceUnreachable, String::default())
        }
    }
}
