use crate::{DeviceId, Result};

use alice::{ModeFunction::*, StateCapability, StateDevice, StateProperty, ToggleFunction::Pause};
use serde_json::Value;
use topics::{ElisaState, Room};

use serde::Deserialize;

pub fn prepare_vacuum_updates(state: ElisaState, payload: Value) -> Result<Vec<StateDevice>> {
    match state {
        ElisaState::Status => {
            let status: Status = serde_json::from_value(payload)?;

            let mut devices = vec![];

            for room in Room::all_rooms() {
                let device_id = DeviceId::vacuum_cleaner_at_room(room);

                let properties = vec![StateProperty::battery_level(status.battery_level.into())];

                let capabilities = vec![
                    StateCapability::on_off(status.is_enabled),
                    StateCapability::mode(WorkSpeed, status.fan_speed.into()),
                    StateCapability::toggle(Pause, status.is_paused),
                ];

                devices.push(StateDevice::new_with_properties_and_capabilities(
                    device_id.to_string(),
                    properties,
                    capabilities,
                ));
            }

            Ok(devices)
        }
    }
}

#[derive(Debug, Deserialize)]
struct Status {
    battery_level: u8,
    is_enabled: bool,
    is_paused: bool,
    fan_speed: crate::types::VacuumFanSpeed,
}
