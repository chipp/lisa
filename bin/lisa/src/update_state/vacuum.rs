use std::sync::Arc;

use crate::{DeviceId, Result};
use crate::{DeviceType::*, Room};

use log::debug;

use alice::{Mode, ModeFunction, ToggleFunction, UpdateStateCapability, UpdatedDeviceState};
use elisheba::Command as VacuumCommand;
use tokio::sync::Mutex;

use super::prepare_result;

#[derive(Default)]
pub struct VacuumUpdate {
    pub rooms: Vec<Room>,
    pub state: Option<bool>,
    pub work_speed: Option<Mode>,
    pub toggle_pause: Option<bool>,
}

pub async fn update_vacuum<F>(
    update: VacuumUpdate,
    devices: &mut Vec<UpdatedDeviceState>,
    send_command: Arc<Mutex<impl Fn(VacuumCommand) -> F>>,
) where
    F: std::future::Future<Output = Result<()>>,
{
    debug!("update state rooms {:?}", update.rooms);
    debug!("state {:?}", update.state);
    debug!("work_speed {:?}", update.work_speed);
    debug!("toggle pause {:?}", update.toggle_pause);

    if !update.rooms.is_empty() {
        return;
    }

    let set_mode_result;
    let set_state_result;
    let toggle_pause_result;

    {
        let send_command = send_command.clone().lock_owned().await;

        set_mode_result = match update.work_speed {
            Some(mode) => Some(
                send_command(VacuumCommand::SetWorkSpeed {
                    mode: mode.to_string(),
                })
                .await,
            ),
            None => None,
        };

        set_state_result = match update.state {
            Some(true) => {
                let room_ids = update.rooms.iter().map(crate::Room::vacuum_id).collect();

                Some(send_command(VacuumCommand::Start { rooms: room_ids }).await)
            }
            Some(false) => {
                let stop = send_command(VacuumCommand::Stop).await;
                let home = send_command(VacuumCommand::GoHome).await;

                Some(stop.and(home))
            }
            None => None,
        };

        toggle_pause_result = match update.toggle_pause {
            Some(true) => Some(send_command(VacuumCommand::Pause).await),
            Some(false) => Some(send_command(VacuumCommand::Resume).await),
            None => None,
        };
    }

    for room in update.rooms {
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
            _ => return,
        }

        let device_id = DeviceId {
            room,
            device_type: VacuumCleaner,
        };

        devices.push(UpdatedDeviceState::new(device_id.to_string(), capabilities));
    }
}
