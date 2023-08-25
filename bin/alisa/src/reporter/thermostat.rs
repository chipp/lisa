use crate::{Capability, DeviceId, Result, Room};
use alice::{RangeFunction, StateCapability, StateDevice, StateProperty};

pub fn prepare_thermostat_update(
    room: Room,
    capability: Capability,
    payload: &[u8],
) -> Result<StateDevice> {
    let device_id = DeviceId::thermostat_at_room(room);

    match capability {
        Capability::IsEnabled => {
            let value: bool = serde_json::from_slice(payload)?;

            Ok(StateDevice::new_with_capabilities(
                device_id.to_string(),
                vec![StateCapability::on_off(value)],
            ))
        }
        Capability::Temperature => {
            let value: f32 = serde_json::from_slice(payload)?;

            Ok(StateDevice::new_with_capabilities(
                device_id.to_string(),
                vec![StateCapability::range(RangeFunction::Temperature, value)],
            ))
        }
        Capability::CurrentTemperature => {
            let value: f32 = serde_json::from_slice(payload)?;

            Ok(StateDevice::new_with_properties(
                device_id.to_string(),
                vec![StateProperty::temperature(value)],
            ))
        }
        Capability::FanSpeed | Capability::Mode => unreachable!(),
    }
}
