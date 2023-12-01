use alice::{Mode, ModeFunction, StateCapability, StateDevice};
use transport::elizabeth::{Capability, CurrentState, State};
use transport::DeviceId;

pub fn prepare_recuperator_update(state: State) -> Option<StateDevice> {
    let state_capability = match state.capability {
        Capability::IsEnabled(value) => StateCapability::on_off(value),
        Capability::FanSpeed(fan_speed) => {
            StateCapability::mode(ModeFunction::FanSpeed, map_fan_speed(fan_speed))
        }
        Capability::Temperature(_) | Capability::CurrentTemperature(_) => {
            return None;
        }
    };

    let device_id = DeviceId::recuperator_at_room(state.room);

    Some(StateDevice::new_with_capabilities(
        device_id,
        vec![state_capability],
    ))
}

pub fn prepare_recuperator_current_state(state: CurrentState) -> StateDevice {
    let device_id = DeviceId::recuperator_at_room(state.room);

    let state_capabilities = state
        .capabilities
        .into_iter()
        .filter_map(|c| match c {
            Capability::IsEnabled(value) => Some(StateCapability::on_off(value)),
            Capability::FanSpeed(fan_speed) => Some(StateCapability::mode(
                ModeFunction::FanSpeed,
                map_fan_speed(fan_speed),
            )),
            Capability::Temperature(_) | Capability::CurrentTemperature(_) => None,
        })
        .collect::<Vec<_>>();

    StateDevice::new_with_capabilities(device_id, state_capabilities)
}

fn map_fan_speed(speed: transport::elizabeth::FanSpeed) -> Mode {
    match speed {
        transport::elizabeth::FanSpeed::Low => Mode::Low,
        transport::elizabeth::FanSpeed::Medium => Mode::Medium,
        transport::elizabeth::FanSpeed::High => Mode::High,
    }
}
