use alice::{RangeFunction, StateCapability, StateDevice, StateProperty};
use topics::{ElizabethState, Room};

use crate::{DeviceId, Result};

use serde_json::Value;

pub fn prepare_thermostat_update(
    room: Option<Room>,
    state: ElizabethState,
    payload: Value,
) -> Result<StateDevice> {
    // TODO: throw an error

    let room = room.unwrap();
    let device_id = DeviceId::thermostat_at_room(room);

    match state {
        ElizabethState::IsEnabled => {
            let value: bool = serde_json::from_value(payload)?;

            Ok(StateDevice::new_with_capabilities(
                device_id.to_string(),
                vec![StateCapability::on_off(value)],
            ))
        }
        ElizabethState::Temperature => {
            let value: f32 = serde_json::from_value(payload)?;

            Ok(StateDevice::new_with_capabilities(
                device_id.to_string(),
                vec![StateCapability::range(RangeFunction::Temperature, value)],
            ))
        }
        ElizabethState::CurrentTemperature => {
            let value: f32 = serde_json::from_value(payload)?;

            Ok(StateDevice::new_with_properties(
                device_id.to_string(),
                vec![StateProperty::temperature(value)],
            ))
        }
        ElizabethState::FanSpeed | ElizabethState::Mode => unreachable!(),
    }
}
