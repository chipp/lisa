use std::{str::FromStr, sync::Arc};

use crate::DeviceId;
use crate::DeviceType::*;

use alice::{
    ModeFunction, StateCapability, StateUpdateResult, UpdateStateCapability, UpdateStateDevice,
    UpdatedDeviceState,
};
use elisheva::Command;
use log::{debug, info};
use tokio::sync::Mutex;

pub async fn update_devices_state<'a>(
    devices: Vec<UpdateStateDevice<'a>>,
    cmd: Arc<Mutex<crate::Commander>>,
) -> Vec<UpdatedDeviceState> {
    let mut rooms = vec![];
    let mut state = None;
    let mut cleanup_mode = None;

    for device in devices.into_iter() {
        if let Ok(DeviceId { room, device_type }) = DeviceId::from_str(device.id) {
            if device_type != VacuumCleaner {
                // TODO: ??
                continue;
            }

            rooms.push(room);

            if state != None && cleanup_mode != None {
                continue;
            }

            for capability in device.capabilities.into_iter() {
                match capability {
                    StateCapability::OnOff { value } => state = Some(value),
                    StateCapability::Mode {
                        function: ModeFunction::WorkSpeed,
                        mode,
                    } => cleanup_mode = Some(mode),
                }
            }
        }
    }

    info!(
        "update state rooms {:?}\nstate {:?}\ncleanup_mode {:?}",
        rooms, state, cleanup_mode
    );

    if rooms.is_empty() {
        return vec![];
    }

    {
        let mut cmd = cmd.lock_owned().await;

        if let Some(ref cleanup_mode) = cleanup_mode {
            cmd.send_command(Command::SetMode {
                mode: cleanup_mode.to_string(),
            })
            .await
            .unwrap();
        }

        match state {
            Some(true) => {
                let room_ids = rooms.iter().map(crate::Room::id).collect();

                cmd.send_command(Command::Start { rooms: room_ids })
                    .await
                    .unwrap();
            }
            Some(false) => {
                cmd.send_command(Command::Stop).await.unwrap();
                cmd.send_command(Command::GoHome).await.unwrap();
            }
            None => (),
        }
    }

    let mut devices = vec![];

    for room in rooms {
        let capabilities;

        match (&state, &cleanup_mode) {
            (Some(state), Some(mode)) => {
                debug!("room: {}, state: {}, mode: {}", room, state, mode);

                capabilities = vec![
                    UpdateStateCapability::on_off(StateUpdateResult::Ok),
                    UpdateStateCapability::mode(ModeFunction::WorkSpeed, StateUpdateResult::Ok),
                ];
            }
            (Some(state), None) => {
                debug!("room: {}, state: {}", room, state);

                capabilities = vec![UpdateStateCapability::on_off(StateUpdateResult::Ok)];
            }
            (None, Some(mode)) => {
                debug!("room: {}, mode: {}", room, mode);

                capabilities = vec![UpdateStateCapability::mode(
                    ModeFunction::WorkSpeed,
                    StateUpdateResult::Ok,
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
