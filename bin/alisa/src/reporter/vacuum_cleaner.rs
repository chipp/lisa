use crate::{DeviceId, Result};

use alice::{
    Mode, ModeFunction::*, StateCapability, StateDevice, StateProperty, ToggleFunction::Pause,
};
use transport::elisa::State;
use transport::Room;

pub fn prepare_vacuum_updates(state: State) -> Result<Vec<StateDevice>> {
    let mut devices = vec![];

    for room in Room::all_rooms() {
        let device_id = DeviceId::vacuum_cleaner_at_room(room);

        let properties = vec![StateProperty::battery_level(state.battery_level.into())];

        let capabilities = vec![
            StateCapability::on_off(state.is_enabled),
            StateCapability::mode(WorkSpeed, map_work_speed(state.work_speed)),
            StateCapability::toggle(Pause, state.is_paused),
        ];

        devices.push(StateDevice::new_with_properties_and_capabilities(
            device_id.to_string(),
            properties,
            capabilities,
        ));
    }

    Ok(devices)
}

fn map_work_speed(speed: transport::elisa::WorkSpeed) -> Mode {
    match speed {
        transport::elisa::WorkSpeed::Silent => Mode::Quiet,
        transport::elisa::WorkSpeed::Standard => Mode::Normal,
        transport::elisa::WorkSpeed::Medium => Mode::Medium,
        transport::elisa::WorkSpeed::Turbo => Mode::Turbo,
    }
}
