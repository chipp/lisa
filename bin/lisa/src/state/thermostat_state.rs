use crate::{DeviceId, Room};
use alice::{RangeFunction, StateCapability, StateDevice, StateProperty};

use super::State;

#[derive(PartialEq)]
pub struct ThermostatState {
    room: Room,
    is_enabled: bool,
    room_temperature: f32,
    target_temperature: f32,

    is_enabled_modified: bool,
    room_temperature_modified: bool,
    target_temperature_modified: bool,
}

impl ThermostatState {
    pub fn new(room: Room) -> ThermostatState {
        ThermostatState {
            room,
            is_enabled: false,
            room_temperature: 0.0,
            target_temperature: 0.0,
            is_enabled_modified: false,
            room_temperature_modified: false,
            target_temperature_modified: false,
        }
    }

    pub fn room(&self) -> Room {
        self.room
    }

    pub fn target_temperature(&self) -> f32 {
        self.target_temperature
    }

    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        if self.is_enabled != is_enabled {
            self.is_enabled = is_enabled;
            self.is_enabled_modified = true;
        }
    }

    pub fn set_room_temperature(&mut self, room_temperature: f32) {
        self.room_temperature = room_temperature;
        self.room_temperature_modified = true;
    }

    pub fn set_target_temperature(&mut self, target_temperature: f32) {
        if self.target_temperature != target_temperature {
            self.target_temperature = target_temperature;
            self.target_temperature_modified = true;
        }
    }
}

impl State for ThermostatState {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>) {
        let properties = self.properties(true);

        if properties.is_empty() {
            return;
        }

        let device_id = DeviceId::thermostat_at_room(self.room);

        devices.push(StateDevice::new_with_properties(
            device_id.to_string(),
            properties,
        ));
    }

    fn properties(&self, only_updated: bool) -> Vec<StateProperty> {
        if !only_updated || (only_updated && self.room_temperature_modified) {
            vec![StateProperty::temperature(self.room_temperature)]
        } else {
            vec![]
        }
    }

    fn capabilities(&self, only_updated: bool) -> Vec<StateCapability> {
        let mut capabilities = vec![];

        if !only_updated || (only_updated && self.is_enabled_modified) {
            capabilities.push(StateCapability::on_off(self.is_enabled));
        }

        if !only_updated || (only_updated && self.target_temperature_modified) {
            capabilities.push(StateCapability::range(
                RangeFunction::Temperature,
                self.target_temperature,
            ));
        }

        capabilities
    }

    fn reset_modified(&mut self) {
        self.is_enabled_modified = false;
        self.room_temperature_modified = false;
        self.target_temperature_modified = false;
    }
}
