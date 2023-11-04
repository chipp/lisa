use std::fmt;
use std::str::FromStr;

use serde::de::{self, value, Unexpected};

use topics::{Device, Room};

#[derive(Debug)]
pub struct DeviceId {
    pub room: Room,
    pub device: Device,
}

impl DeviceId {
    pub fn recuperator_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device: Device::Recuperator,
        }
    }

    pub fn temperature_sensor_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device: Device::TemperatureSensor,
        }
    }

    pub fn thermostat_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device: Device::Thermostat,
        }
    }

    pub fn vacuum_cleaner_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device: Device::VacuumCleaner,
        }
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.device, self.room)
    }
}

impl FromStr for DeviceId {
    type Err = value::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        // TODO: refactor with split_once
        let mut parts = id.splitn(2, '/');

        let device_type = parts
            .next()
            .ok_or_else(|| de::Error::invalid_value(Unexpected::Str(id), &"device_type/room"))?;

        let device_type = Device::from_str(device_type).map_err(|err| {
            de::Error::invalid_value(Unexpected::Str(device_type), &err.to_string().as_str())
        })?;

        let room = parts
            .next()
            .ok_or_else(|| de::Error::invalid_value(Unexpected::Str(id), &"device_type/room"))?;

        let room = Room::from_str(room).map_err(|err| {
            de::Error::invalid_value(Unexpected::Str(room), &err.to_string().as_str())
        })?;

        Ok(DeviceId {
            device: device_type,
            room,
        })
    }
}
