use alice::StateProperty;

#[derive(Default, PartialEq)]
pub struct SensorState {
    temperature: u16,
    humidity: u16,
    battery: u8,

    temperature_modified: bool,
    humidity_modified: bool,
    battery_modified: bool,
}

impl SensorState {
    pub fn set_temperature(&mut self, temperature: u16) {
        if self.temperature != temperature {
            self.temperature = temperature;
            self.temperature_modified = true;
        }
    }

    pub fn set_humidity(&mut self, humidity: u16) {
        if self.humidity != humidity {
            self.humidity = humidity;
            self.humidity_modified = true;
        }
    }

    pub fn set_battery(&mut self, battery: u8) {
        if self.battery != battery {
            self.battery = battery;
            self.battery_modified = true;
        }
    }
}

impl SensorState {
    pub fn properties(&self, only_updated: bool) -> Vec<StateProperty> {
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

    pub fn reset_modified(&mut self) {
        self.temperature_modified = false;
        self.humidity_modified = false;
        self.battery_modified = false;
    }
}
