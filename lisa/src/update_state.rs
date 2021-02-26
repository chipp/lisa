use std::str::FromStr;

use crate::DeviceId;
use crate::DeviceType::*;

use alice::{
    ModeFunction, StateCapability, StateUpdateResult, UpdateStateCapability, UpdateStateDevice,
    UpdatedDeviceState,
};
use log::debug;

pub fn update_devices_state(devices: Vec<UpdateStateDevice>) -> Vec<UpdatedDeviceState> {
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
                        function: ModeFunction::CleanupMode,
                        mode,
                    } => cleanup_mode = Some(mode),
                }
            }
        }
    }

    if rooms.is_empty() {
        return vec![];
    }

    let mut devices = vec![];

    for room in rooms {
        let capabilities;

        match (&state, &cleanup_mode) {
            (Some(state), Some(mode)) => {
                debug!("room: {}, state: {}, mode: {:?}", room, state, mode);

                capabilities = vec![
                    UpdateStateCapability::on_off(StateUpdateResult::Ok),
                    UpdateStateCapability::mode(ModeFunction::CleanupMode, StateUpdateResult::Ok),
                ];
            }
            (Some(state), None) => {
                debug!("room: {}, state: {}", room, state);

                capabilities = vec![UpdateStateCapability::on_off(StateUpdateResult::Ok)];
            }
            (None, Some(mode)) => {
                debug!("room: {}, mode: {:?}", room, mode);

                capabilities = vec![UpdateStateCapability::mode(
                    ModeFunction::CleanupMode,
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
