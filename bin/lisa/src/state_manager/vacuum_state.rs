use std::str::FromStr;

use alice::{
    Mode, ModeFunction::WorkSpeed, StateCapability, StateDevice, StateProperty,
    ToggleFunction::Pause,
};

use crate::{DeviceId, Room};

use super::{reportable_property::ReportableProperty, State};

#[derive(Default, PartialEq)]
pub struct VacuumState {
    is_enabled: ReportableProperty<bool>,
    battery: ReportableProperty<u8>,
    work_speed: ReportableProperty<Mode>,
    is_paused: ReportableProperty<bool>,
}

impl VacuumState {
    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        self.is_enabled.set_value(is_enabled, false);
    }

    pub fn set_battery(&mut self, battery: u8) {
        self.battery.set_value(battery, true);
    }

    pub fn set_work_speed(&mut self, work_speed: String) {
        let vacuum_work_speed = Mode::from_str(&work_speed).unwrap();
        self.work_speed.set_value(vacuum_work_speed, false);
    }

    pub fn set_is_paused(&mut self, is_paused: bool) {
        self.is_paused.set_value(is_paused, false);
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
        if !only_updated || (only_updated && self.battery.modified()) {
            vec![StateProperty::battery_level(self.battery.get_value() as f32)]
        } else {
            vec![]
        }
    }

    fn capabilities(&self, only_updated: bool) -> Vec<StateCapability> {
        let mut capabilities = vec![];

        if !only_updated || (only_updated && self.is_enabled.modified()) {
            capabilities.push(StateCapability::on_off(self.is_enabled.get_value()));
        }

        if !only_updated || (only_updated && self.work_speed.modified()) {
            capabilities.push(StateCapability::mode(
                WorkSpeed,
                self.work_speed.get_value(),
            ));
        }

        if !only_updated || (only_updated && self.is_paused.modified()) {
            capabilities.push(StateCapability::toggle(Pause, self.is_paused.get_value()));
        }

        capabilities
    }

    fn reset_modified(&mut self) {
        self.is_enabled.reset_modified();
        self.battery.reset_modified();
        self.work_speed.reset_modified();
        self.is_paused.reset_modified();
    }
}
