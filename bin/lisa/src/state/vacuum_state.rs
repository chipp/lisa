use std::str::FromStr;

use alice::{
    Mode, ModeFunction::WorkSpeed, StateCapability, StateDevice, StateProperty,
    ToggleFunction::Pause,
};

use crate::{DeviceId, Room};

use super::State;

#[derive(Default, PartialEq)]
pub struct VacuumState {
    is_enabled: bool,
    battery: u8,
    work_speed: Mode,
    is_paused: bool,

    is_enabled_modified: bool,
    battery_modified: bool,
    work_speed_modified: bool,
    is_paused_modified: bool,
}

impl VacuumState {
    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        if self.is_enabled != is_enabled {
            self.is_enabled = is_enabled;
            self.is_enabled_modified = true;
        }
    }

    pub fn set_battery(&mut self, battery: u8) {
        self.battery = battery;
        self.battery_modified = true;
    }

    pub fn set_work_speed(&mut self, work_speed: String) {
        let vacuum_work_speed = Mode::from_str(&work_speed).unwrap();
        if self.work_speed != vacuum_work_speed {
            self.work_speed = vacuum_work_speed;
            self.work_speed_modified = true;
        }
    }

    pub fn set_is_paused(&mut self, is_paused: bool) {
        if self.is_paused != is_paused {
            self.is_paused = is_paused;
            self.is_paused_modified = true;
        }
    }
}

impl State for VacuumState {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>) {
        let properties = self.properties(true);
        let capabilities = self.capabilities(true);

        if properties.is_empty() && capabilities.is_empty() {
            return;
        }

        for room in Room::all_rooms() {
            let device_id = DeviceId::vacuum_cleaner_at_room(*room);

            devices.push(StateDevice::new_with_properties_and_capabilities(
                device_id.to_string(),
                properties.clone(),
                capabilities.clone(),
            ));
        }
    }

    fn properties(&self, only_updated: bool) -> Vec<StateProperty> {
        if !only_updated || (only_updated && self.battery_modified) {
            vec![StateProperty::battery_level(self.battery as f32)]
        } else {
            vec![]
        }
    }

    fn capabilities(&self, only_updated: bool) -> Vec<StateCapability> {
        let mut capabilities = vec![];

        if !only_updated || (only_updated && self.is_enabled_modified) {
            capabilities.push(StateCapability::on_off(self.is_enabled));
        }

        if !only_updated || (only_updated && self.work_speed_modified) {
            capabilities.push(StateCapability::mode(WorkSpeed, self.work_speed));
        }

        if !only_updated || (only_updated && self.is_paused_modified) {
            capabilities.push(StateCapability::toggle(Pause, self.is_paused));
        }

        capabilities
    }

    fn reset_modified(&mut self) {
        self.is_enabled_modified = false;
        self.battery_modified = false;
        self.work_speed_modified = false;
        self.is_paused_modified = false;
    }
}
