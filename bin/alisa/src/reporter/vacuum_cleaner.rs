use alice::{
    Mode, ModeFunction::*, StateCapability, StateDevice, StateProperty, ToggleFunction::Pause,
};
use transport::elisa::State;
use transport::{DeviceId, Room};

pub fn prepare_vacuum_updates(state: State) -> Vec<StateDevice> {
    let mut devices = vec![];
    let all_rooms = vacuum_rooms();

    let state_rooms: &[Room] = if state.is_enabled && !state.rooms.is_empty() {
        &state.rooms
    } else {
        &all_rooms
    };

    for room in all_rooms {
        let device_id = DeviceId::vacuum_cleaner_at_room(room);

        let properties = vec![StateProperty::battery_level(state.battery_level.into())];

        let mut capabilities = vec![StateCapability::mode(
            WorkSpeed,
            map_work_speed(state.work_speed),
        )];

        capabilities.push(StateCapability::mode(
            CleanupMode,
            map_cleanup_mode(state.cleanup_mode),
        ));

        if state_rooms.contains(&room) {
            capabilities.push(StateCapability::on_off(state.is_enabled));
            capabilities.push(StateCapability::toggle(Pause, state.is_paused));
        } else {
            capabilities.push(StateCapability::on_off(!state.is_enabled));
            capabilities.push(StateCapability::toggle(Pause, !state.is_paused));
        }

        devices.push(StateDevice::new_with_properties_and_capabilities(
            device_id,
            properties,
            capabilities,
        ));
    }

    devices
}

fn vacuum_rooms() -> [transport::Room; 6] {
    [
        Room::Bedroom,
        Room::Corridor,
        Room::Hallway,
        Room::HomeOffice,
        Room::Kitchen,
        Room::LivingRoom,
    ]
}

fn map_work_speed(speed: transport::elisa::WorkSpeed) -> Mode {
    match speed {
        transport::elisa::WorkSpeed::Silent => Mode::Quiet,
        transport::elisa::WorkSpeed::Standard => Mode::Normal,
        transport::elisa::WorkSpeed::Medium => Mode::Medium,
        transport::elisa::WorkSpeed::Turbo => Mode::Turbo,
    }
}

fn map_cleanup_mode(mode: transport::elisa::CleanupMode) -> Mode {
    match mode {
        transport::elisa::CleanupMode::DryCleaning => Mode::DryCleaning,
        transport::elisa::CleanupMode::WetCleaning => Mode::WetCleaning,
        transport::elisa::CleanupMode::MixedCleaning => Mode::MixedCleaning,
    }
}
