use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Packet, PacketContent};

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorData {
    pub room: String,
    pub temperature: f32,
    pub humidity: f32,
    pub battery: u8,
}

impl PacketContent for SensorData {
    fn to_packet(self) -> Packet {
        Packet::SensorData(self)
    }
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
