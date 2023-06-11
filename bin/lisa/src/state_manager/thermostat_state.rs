use crate::{DeviceId, Room};

use super::{reportable_property::ReportableProperty, State};
use alice::{RangeFunction, StateCapability, StateDevice, StateProperty};
use log::{debug, info};

#[derive(PartialEq)]
pub struct ThermostatState {
    room: Room,
    is_enabled: ReportableProperty<bool>,
    room_temperature: ReportableProperty<f32>,
    target_temperature: ReportableProperty<f32>,
}

impl ThermostatState {
    pub fn new(room: Room) -> ThermostatState {
        ThermostatState {
            room,
            is_enabled: ReportableProperty::default(),
            room_temperature: ReportableProperty::default(),
            target_temperature: ReportableProperty::default(),
        }
    }

    pub fn room(&self) -> Room {
        self.room
    }

    pub fn target_temperature(&self) -> f32 {
        self.target_temperature.get_value()
    }

    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        if self.is_enabled.set_value(is_enabled, false) {
            info!("set thermostat {:?} is enabled = {}", self.room, is_enabled);
        }
    }

    pub fn set_room_temperature(&mut self, room_temperature: f32) {
        if self.room_temperature.set_value(room_temperature, true) {
            debug!(
                "set thermostat {:?} room temperature = {}",
                self.room, room_temperature
            );
        }
    }

    pub fn set_target_temperature(&mut self, target_temperature: f32) {
        if self.target_temperature.set_value(target_temperature, false) {
            info!(
                "set thermostat {:?} target temperature = {}",
                self.room, target_temperature
            );
        }
    }
}

impl State for ThermostatState {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>) {
        let properties = self.properties(true);
        let capabilities = self.capabilities(true);

        if properties.is_empty() && capabilities.is_empty() {
            return;
        }

        let device_id = DeviceId::thermostat_at_room(self.room);

        devices.push(StateDevice::new_with_properties_and_capabilities(
            device_id.to_string(),
            properties,
            capabilities,
        ));
    }

    fn properties(&self, only_updated: bool) -> Vec<StateProperty> {
        if !only_updated || (only_updated && self.room_temperature.modified()) {
            vec![StateProperty::temperature(
                self.room_temperature.get_value(),
            )]
        } else {
            vec![]
        }
    }

    fn capabilities(&self, only_updated: bool) -> Vec<StateCapability> {
        let mut capabilities = vec![];

        if !only_updated || (only_updated && self.is_enabled.modified()) {
            capabilities.push(StateCapability::on_off(self.is_enabled.get_value()));
        }

        if !only_updated || (only_updated && self.target_temperature.modified()) {
            capabilities.push(StateCapability::range(
                RangeFunction::Temperature,
                self.target_temperature.get_value(),
            ));
        }

        capabilities
    }

    fn reset_modified(&mut self) {
        self.is_enabled.reset_modified();
        self.room_temperature.reset_modified();
        self.target_temperature.reset_modified();
    }
}
