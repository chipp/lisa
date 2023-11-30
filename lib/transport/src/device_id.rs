use std::fmt;
use std::str::FromStr;

use serde::{
    de::{self, Unexpected},
    Deserialize, Serialize,
};

use crate::{DeviceType, Room};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

impl Serialize for DeviceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}/{}", self.device_type, self.room).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DeviceIdVisitor;

        impl<'de> de::Visitor<'de> for DeviceIdVisitor {
            type Value = DeviceId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("device_type/room")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut parts = value.splitn(2, '/');

                let device_type = parts.next().ok_or_else(|| {
                    de::Error::invalid_value(Unexpected::Str(value), &"device_type/room")
                })?;

                let device_type = DeviceType::from_str(device_type).map_err(|err| {
                    de::Error::invalid_value(
                        Unexpected::Str(device_type),
                        &err.to_string().as_str(),
                    )
                })?;

                let room = parts.next().ok_or_else(|| {
                    de::Error::invalid_value(Unexpected::Str(value), &"device_type/room")
                })?;

                let room = Room::from_str(room).map_err(|err| {
                    de::Error::invalid_value(Unexpected::Str(room), &err.to_string().as_str())
                })?;

                Ok(DeviceId { device_type, room })
            }
        }

        deserializer.deserialize_str(DeviceIdVisitor)
    }
}
