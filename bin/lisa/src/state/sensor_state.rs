use crate::{DeviceId, Room};
use alice::{StateCapability, StateDevice, StateProperty};

use super::State;

#[derive(PartialEq)]
pub struct SensorState {
    room: Room,
    temperature: u16,
    humidity: u16,
    battery: u8,

    temperature_modified: bool,
    humidity_modified: bool,
    battery_modified: bool,
}

impl SensorState {
    pub fn new(room: Room) -> SensorState {
        SensorState {
            room,
            temperature: 0,
            humidity: 0,
            battery: 0,
            temperature_modified: false,
            humidity_modified: false,
            battery_modified: false,
        }
    }

    pub fn set_temperature(&mut self, temperature: u16) {
        self.temperature = temperature;
        self.temperature_modified = true;
    }

    pub fn set_humidity(&mut self, humidity: u16) {
        self.humidity = humidity;
        self.humidity_modified = true;
    }

    pub fn set_battery(&mut self, battery: u8) {
        self.battery = battery;
        self.battery_modified = true;
    }
}

impl State for SensorState {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>) {
        let properties = self.properties(true);

        if properties.is_empty() {
            return;
        }

        let device_id = DeviceId::temperature_sensor_at_room(self.room);

        devices.push(StateDevice::new_with_properties(
            device_id.to_string(),
            properties,
        ));
    }

    fn properties(&self, only_updated: bool) -> Vec<StateProperty> {
        let mut properties = vec![];

        if !only_updated || (only_updated && self.temperature_modified) {
            properties.push(StateProperty::temperature(self.temperature as f32 / 10.0))
        }

        if !only_updated || (only_updated && self.humidity_modified) {
            properties.push(StateProperty::humidity(self.humidity as f32 / 10.0))
        }

        if !only_updated || (only_updated && self.battery_modified) {
            properties.push(StateProperty::battery_level(self.battery as f32))
        }

        properties
    }

    fn capabilities(&self, _: bool) -> Vec<StateCapability> {
        vec![]
    }

    fn reset_modified(&mut self) {
        self.temperature_modified = false;
        self.humidity_modified = false;
        self.battery_modified = false;
    }
}
