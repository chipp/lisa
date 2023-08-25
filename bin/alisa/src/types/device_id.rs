use std::fmt;
use std::str::FromStr;

use serde::de::{self, value, Unexpected};

use crate::types::{DeviceType, Room};

#[derive(Debug)]
pub struct DeviceId {
    pub room: Room,
    pub device_type: DeviceType,
}

impl DeviceId {
    pub fn recuperator_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device_type: DeviceType::Recuperator,
        }
    }

    pub fn temperature_sensor_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device_type: DeviceType::TemperatureSensor,
        }
    }

    pub fn thermostat_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device_type: DeviceType::Thermostat,
        }
    }

    pub fn vacuum_cleaner_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device_type: DeviceType::VacuumCleaner,
        }
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            self.device_type.to_string(),
            self.room.to_string()
        )
    }
}

impl FromStr for DeviceId {
    type Err = value::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        let mut parts = id.splitn(2, "/");

        let device_type = parts
            .next()
            .ok_or_else(|| de::Error::invalid_value(Unexpected::Str(id), &"device_type/room"))?;

        let device_type = DeviceType::from_str(device_type).map_err(|err| {
            de::Error::invalid_value(Unexpected::Str(device_type), &err.to_string().as_str())
        })?;

        let room = parts
            .next()
            .ok_or_else(|| de::Error::invalid_value(Unexpected::Str(id), &"device_type/room"))?;

        let room = Room::from_str(room).map_err(|err| {
            de::Error::invalid_value(Unexpected::Str(room), &err.to_string().as_str())
        })?;

        Ok(DeviceId { device_type, room })
    }
}
