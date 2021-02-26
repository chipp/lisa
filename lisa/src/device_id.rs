use std::fmt;
use std::str::FromStr;

use serde::de::{self, value, IntoDeserializer, MapAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

use super::device_type::DeviceType;
use super::Room;

#[derive(Debug)]
pub struct DeviceId {
    pub room: Room,
    pub device_type: DeviceType,
}

impl DeviceId {
    pub fn vacuum_cleaner_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device_type: DeviceType::VacuumCleaner,
        }
    }

    pub fn temperature_sensor_at_room(room: Room) -> DeviceId {
        DeviceId {
            room,
            device_type: DeviceType::TemperatureSensor,
        }
    }
}

impl<'de> Deserialize<'de> for DeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("DeviceId", FIELDS, DeviceIdVisitor)
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field {
    Id,
}

const FIELDS: &[&str] = &["id"];

struct DeviceIdVisitor;

impl<'de> Visitor<'de> for DeviceIdVisitor {
    type Value = DeviceId;

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut id = None;

        while let Some(field) = map.next_key()? {
            match field {
                Field::Id => {
                    id = Some(map.next_value()?);
                }
            }
        }

        let id: &str = id.ok_or_else(|| de::Error::missing_field("id"))?;
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

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string with device type and room")
    }
}
