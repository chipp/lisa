use alice::{Mode, ModeFunction, RangeFunction, StateCapability, StateDevice, StateProperty};
use transport::elzhbieta::{Mode as AcMode, State};
use transport::DeviceId;

pub fn prepare_air_conditioner_state(state: State) -> StateDevice {
    let device_id = DeviceId::air_conditioner_at_room(state.room);

    StateDevice::new_with_properties_and_capabilities(
        device_id,
        vec![
            StateProperty::temperature(state.current_temperature),
            StateProperty::humidity(state.humidity.into()),
        ],
        vec![
            StateCapability::on_off(state.is_enabled),
            StateCapability::mode(ModeFunction::Thermostat, map_mode(state.mode)),
            StateCapability::range(RangeFunction::Temperature, state.target_temperature),
        ],
    )
}

fn map_mode(mode: AcMode) -> Mode {
    match mode {
        AcMode::Cool => Mode::Cool,
        AcMode::Heat => Mode::Heat,
        AcMode::Dry => Mode::Dry,
        AcMode::FanOnly => Mode::FanOnly,
        AcMode::Auto => Mode::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};
    use transport::Room;

    #[test]
    fn test_prepare_air_conditioner_state() {
        let state = prepare_air_conditioner_state(State {
            room: Room::Bedroom,
            is_enabled: true,
            mode: AcMode::Cool,
            target_temperature: 22.0,
            current_temperature: 27.0,
            humidity: 40,
        });

        assert_eq!(
            to_value(state).unwrap(),
            json!({
                "id": "air_conditioner/bedroom",
                "capabilities": [
                    {"type": "devices.capabilities.on_off", "state": {"instance": "on", "value": true}},
                    {"type": "devices.capabilities.mode", "state": {"instance": "thermostat", "value": "cool"}},
                    {"type": "devices.capabilities.range", "state": {"instance": "temperature", "value": 22.0}}
                ],
                "properties": [
                    {"type": "devices.properties.float", "state": {"instance": "temperature", "value": 27.0}},
                    {"type": "devices.properties.float", "state": {"instance": "humidity", "value": 40.0}}
                ]
            })
        );
    }
}
