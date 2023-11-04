use alice::{RangeFunction, StateCapability, StateDevice, StateProperty};
use transport::elizabeth::{Capability, State};

use crate::{DeviceId, Result};

pub fn prepare_thermostat_update(state: State) -> Result<StateDevice> {
    let device_id = DeviceId::thermostat_at_room(state.room);

    match state.capability {
        Capability::IsEnabled(value) => Ok(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![StateCapability::on_off(value)],
        )),
        Capability::Temperature(value) => Ok(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![StateCapability::range(RangeFunction::Temperature, value)],
        )),
        Capability::CurrentTemperature(value) => Ok(StateDevice::new_with_properties(
            device_id.to_string(),
            vec![StateProperty::temperature(value)],
        )),
        Capability::FanSpeed(_) => unreachable!(),
    }
}
