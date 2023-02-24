use crate::{DeviceId, Room};
use alice::{Mode, ModeFunction, StateCapability, StateDevice, StateProperty};
use alisa::FanSpeed;
use log::info;

use super::{reportable_property::ReportableProperty, State};

#[derive(PartialEq)]
pub struct RecuperatorState {
    is_enabled: ReportableProperty<bool>,
    fan_speed: ReportableProperty<FanSpeed>,
}

impl RecuperatorState {
    pub fn new() -> RecuperatorState {
        RecuperatorState {
            is_enabled: ReportableProperty::new(false),
            fan_speed: ReportableProperty::new(FanSpeed::Low),
        }
    }

    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        if self.is_enabled.set_value(is_enabled, false) {
            info!("set recuperator is enabled = {}", is_enabled);
        }
    }

    pub fn set_fan_speed(&mut self, fan_speed: FanSpeed) {
        if self.fan_speed.set_value(fan_speed, false) {
            info!("set recuperator fan speed = {:?}", fan_speed);
        }
    }
}

impl State for RecuperatorState {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>) {
        let capabilities = self.capabilities(true);

        if capabilities.is_empty() {
            return;
        }

        let device_id = DeviceId::recuperator_at_room(Room::LivingRoom);

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

        if !only_updated || (only_updated && self.is_enabled.modified()) {
            capabilities.push(StateCapability::on_off(self.is_enabled.get_value()));
        }

        if !only_updated || (only_updated && self.fan_speed.modified()) {
            capabilities.push(StateCapability::mode(
                ModeFunction::FanSpeed,
                map_fan_speed(&self.fan_speed.get_value()),
            ));
        }

        capabilities
    }

    fn reset_modified(&mut self) {
        self.is_enabled.reset_modified();
        self.fan_speed.reset_modified();
    }
}
fn map_fan_speed(fan_speed: &FanSpeed) -> Mode {
    match fan_speed {
        FanSpeed::Low => Mode::Low,
        FanSpeed::Medium => Mode::Medium,
        FanSpeed::High => Mode::High,
    }
}
