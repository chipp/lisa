use alice::{RangeFunction, StateCapability, StateDevice, StateProperty};
use transport::elizabeth::{Capability, CurrentState, State};
use transport::DeviceId;

pub fn prepare_thermostat_update(state: State) -> Option<StateDevice> {
    let device_id = DeviceId::thermostat_at_room(state.room);

    match state.capability {
        Capability::IsEnabled(value) => Some(StateDevice::new_with_capabilities(
            device_id,
            vec![StateCapability::on_off(value)],
        )),
        Capability::Temperature(value) => Some(StateDevice::new_with_capabilities(
            device_id,
            vec![StateCapability::range(RangeFunction::Temperature, value)],
        )),
        Capability::CurrentTemperature(value) => Some(StateDevice::new_with_properties(
            device_id,
            vec![StateProperty::temperature(value)],
        )),
        Capability::FanSpeed(_) => None,
    }
}

pub fn prepare_thermostat_current_state(state: CurrentState) -> StateDevice {
    let device_id = DeviceId::thermostat_at_room(state.room);

    let state_capabilities = state
        .capabilities
        .iter()
        .filter_map(|c| match c {
            Capability::IsEnabled(value) => Some(StateCapability::on_off(*value)),
            Capability::Temperature(value) => {
                Some(StateCapability::range(RangeFunction::Temperature, *value))
            }
            Capability::CurrentTemperature(_) | Capability::FanSpeed(_) => None,
        })
        .collect::<Vec<_>>();

    let state_properties = state
        .capabilities
        .into_iter()
        .filter_map(|c| {
            if let Capability::CurrentTemperature(value) = c {
                Some(StateProperty::temperature(value))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    StateDevice::new_with_properties_and_capabilities(
        device_id,
        state_properties,
        state_capabilities,
    )
}
