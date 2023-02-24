use crate::{DeviceId, Room};
use alice::{StateCapability, StateDevice, StateProperty};

use super::{reportable_property::ReportableProperty, State};

#[derive(PartialEq)]
pub struct SensorState {
    room: Room,
    temperature: ReportableProperty<u16>,
    humidity: ReportableProperty<u16>,
    battery: ReportableProperty<u8>,
}

impl SensorState {
    pub fn new(room: Room) -> SensorState {
        SensorState {
            room,
            temperature: ReportableProperty::default(),
            humidity: ReportableProperty::default(),
            battery: ReportableProperty::default(),
        }
    }

    pub fn room(&self) -> Room {
        self.room
    }

    pub fn set_temperature(&mut self, temperature: u16) {
        self.temperature.set_value(temperature, true);
    }

    pub fn set_humidity(&mut self, humidity: u16) {
        self.humidity.set_value(humidity, true);
    }

    pub fn set_battery(&mut self, battery: u8) {
        self.battery.set_value(battery, true);
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

        if !only_updated || (only_updated && self.temperature.modified()) {
            properties.push(StateProperty::temperature(
                self.temperature.get_value() as f32 / 10.0,
            ))
        }

        if !only_updated || (only_updated && self.humidity.modified()) {
            properties.push(StateProperty::humidity(
                self.humidity.get_value() as f32 / 10.0,
            ))
        }

        if !only_updated || (only_updated && self.battery.modified()) {
            properties.push(StateProperty::battery_level(self.battery.get_value() as f32))
        }

        properties
    }

    fn capabilities(&self, _: bool) -> Vec<StateCapability> {
        vec![]
    }

    fn reset_modified(&mut self) {
        self.temperature.reset_modified();
        self.humidity.reset_modified();
        self.battery.reset_modified();
    }
}
