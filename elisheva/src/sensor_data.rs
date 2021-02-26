use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorData {
    pub temperature: f32,
    pub humidity: f32,
    pub battery: u32,
}

impl fmt::Display for SensorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "T: {} / H: {} / B: {}",
            self.temperature, self.humidity, self.battery
        )
    }
}
