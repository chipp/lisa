use serde::{Deserialize, Serialize};

use crate::{Packet, PacketContent};

#[derive(Debug, Deserialize, Serialize)]
pub enum SensorData {
    Temperature {
        room: SensorRoom,
        temperature: u16,
    },
    Humidity {
        room: SensorRoom,
        humidity: u16,
    },
    Battery {
        room: SensorRoom,
        battery: u8,
    },
    TemperatureAndHumidity {
        room: SensorRoom,
        temperature: u16,
        humidity: u16,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SensorRoom {
    Nursery,
    Bedroom,
    LivingRoom,
}

impl SensorData {
    pub fn room(&self) -> &SensorRoom {
        match self {
            SensorData::Temperature {
                room,
                temperature: _,
            } => room,
            SensorData::Humidity { room, humidity: _ } => room,
            SensorData::Battery { room, battery: _ } => room,
            SensorData::TemperatureAndHumidity {
                room,
                temperature: _,
                humidity: _,
            } => room,
        }
    }
}

impl PacketContent for SensorData {
    fn to_packet(self) -> Packet {
        Packet::SensorData(self)
    }
}
