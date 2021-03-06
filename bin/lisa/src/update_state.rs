use std::{str::FromStr, sync::Arc};

use crate::DeviceType::*;
use crate::{DeviceId, Result};

use log::debug;

use alice::{
    ModeFunction, StateCapability, StateUpdateResult, ToggleFunction, UpdateStateCapability,
    UpdateStateDevice, UpdateStateErrorCode, UpdatedDeviceState,
};
use elisheba::Command;
use tokio::sync::Mutex;

pub async fn update_devices_state<'a, F>(
    devices: Vec<UpdateStateDevice<'a>>,
    send_command: Arc<Mutex<impl Fn(Command) -> F>>,
) -> Vec<UpdatedDeviceState>
where
    F: std::future::Future<Output = Result<()>>,
{
    let mut rooms = vec![];
    let mut state = None;
    let mut work_speed = None;
    let mut toggle_pause = None;

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
                    StateCapability::Toggle {
                        function: ToggleFunction::Pause,
                        value,
                    } => toggle_pause = Some(value),
                }
            }
        }
    }

    debug!("update state rooms {:?}", rooms);
    debug!("state {:?}", state);
    debug!("work_speed {:?}", work_speed);
    debug!("toggle pause {:?}", toggle_pause);

    if rooms.is_empty() {
        return vec![];
    }

    let set_mode_result;
    let set_state_result;
    let toggle_pause_result;

    {
        let send_command = send_command.clone().lock_owned().await;

        set_mode_result = match work_speed {
            Some(mode) => Some(
                send_command(Command::SetWorkSpeed {
                    mode: mode.to_string(),
                })
                .await,
            ),
            None => None,
        };

        set_state_result = match state {
            Some(true) => {
                let room_ids = rooms.iter().map(crate::Room::vacuum_id).collect();

                Some(send_command(Command::Start { rooms: room_ids }).await)
            }
            Some(false) => {
                let stop = send_command(Command::Stop).await;
                let home = send_command(Command::GoHome).await;

                Some(stop.and(home))
            }
            None => None,
        };

        toggle_pause_result = match toggle_pause {
            Some(true) => Some(send_command(Command::Pause).await),
            Some(false) => Some(send_command(Command::Resume).await),
            None => None,
        };
    }

    let mut devices = vec![];

    for room in rooms {
        let capabilities;

        match (&set_state_result, &set_mode_result, &toggle_pause_result) {
            (Some(toggle_state_result), Some(set_mode_result), None) => {
                capabilities = vec![
                    UpdateStateCapability::on_off(prepare_result(&toggle_state_result)),
                    UpdateStateCapability::mode(
                        ModeFunction::WorkSpeed,
                        prepare_result(&set_mode_result),
                    ),
                ];
            }
            (Some(set_state_result), None, None) => {
                capabilities = vec![UpdateStateCapability::on_off(prepare_result(
                    &set_state_result,
                ))];
            }
            (None, Some(set_mode_result), None) => {
                capabilities = vec![UpdateStateCapability::mode(
                    ModeFunction::WorkSpeed,
                    prepare_result(&set_mode_result),
                )];
            }
            (None, None, Some(toggle_pause_result)) => {
                capabilities = vec![UpdateStateCapability::toggle(
                    ToggleFunction::Pause,
                    prepare_result(&toggle_pause_result),
                )]
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

fn prepare_result(result: &Result<()>) -> StateUpdateResult {
    match result {
        Ok(_) => StateUpdateResult::ok(),
        Err(_) => {
            // TODO:
            StateUpdateResult::error(UpdateStateErrorCode::DeviceUnreachable, String::default())
        }
    }
}
