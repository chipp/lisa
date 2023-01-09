use crate::{DeviceId, Room};
use alice::{Mode, ModeFunction, StateCapability, StateDevice, StateProperty};
use alisa::FanSpeed;

use super::State;

#[derive(PartialEq)]
pub struct RecuperatorState {
    is_enabled: bool,
    fan_speed: FanSpeed,

    is_enabled_modified: bool,
    fan_speed_modified: bool,
}

impl RecuperatorState {
    pub fn new() -> RecuperatorState {
        RecuperatorState {
            is_enabled: false,
            fan_speed: FanSpeed::Low,
            is_enabled_modified: false,
            fan_speed_modified: false,
        }
    }

    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        if self.is_enabled != is_enabled {
            self.is_enabled = is_enabled;
            self.is_enabled_modified = true;
        }
    }

    pub fn set_fan_speed(&mut self, fan_speed: FanSpeed) {
        if self.fan_speed != fan_speed {
            self.fan_speed = fan_speed;
            self.fan_speed_modified = true;
        }
    }
}

impl State for RecuperatorState {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>) {
        let capabilities = self.capabilities(true);

        if capabilities.is_empty() {
            return;
        }

        let device_id = DeviceId::thermostat_at_room(Room::LivingRoom);

        devices.push(StateDevice::new_with_capabilities(
            device_id.to_string(),
            capabilities,
        ));
    }

    fn properties(&self, _only_updated: bool) -> Vec<StateProperty> {
        vec![]
    }

    fn capabilities(&self, only_updated: bool) -> Vec<StateCapability> {
        let mut capabilities = vec![];

        if !only_updated || (only_updated && self.is_enabled_modified) {
            capabilities.push(StateCapability::on_off(self.is_enabled));
        }

        if !only_updated || (only_updated && self.fan_speed_modified) {
            capabilities.push(StateCapability::mode(
                ModeFunction::FanSpeed,
                map_fan_speed(&self.fan_speed),
            ));
        }

        capabilities
    }

    fn reset_modified(&mut self) {
        self.is_enabled_modified = false;
        self.fan_speed_modified = false;
    }
}
fn map_fan_speed(fan_speed: &FanSpeed) -> Mode {
    match fan_speed {
        FanSpeed::Low => Mode::Low,
        FanSpeed::Medium => Mode::Medium,
        FanSpeed::High => Mode::High,
    }
}
