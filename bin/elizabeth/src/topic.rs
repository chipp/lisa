use std::{fmt, str::FromStr};

use crate::{
    Capability::{self, *},
    DeviceType::{self, *},
    StatePayload,
};
use inspinia::Room::{self, *};
use paho_mqtt::QOS_1;
use serde::{
    de::{value, Error, IntoDeserializer},
    Deserialize, Serialize,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicType {
    State,
    Set,
}

impl fmt::Display for TopicType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TopicType::State => write!(f, "state"),
            TopicType::Set => write!(f, "set"),
        }
    }
}

impl FromStr for TopicType {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

#[derive(Debug)]
pub struct Topic {
    pub topic_type: TopicType,
    pub room: Room,
    pub device_type: DeviceType,
    pub capability: Capability,
}

impl Topic {
    pub const fn state(room: Room, device_type: DeviceType, capability: Capability) -> Self {
        Self {
            topic_type: TopicType::State,
            room,
            device_type,
            capability,
        }
    }

    pub const fn set(room: Room, device_type: DeviceType, capability: Capability) -> Self {
        Self {
            topic_type: TopicType::Set,
            room,
            device_type,
            capability,
        }
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "elizabeth/{}/{}/{}/{}",
            self.topic_type, self.room, self.device_type, self.capability
        )
    }
}

impl FromStr for Topic {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Topic, value::Error> {
        let mut split = s.split('/');

        if let Some("elizabeth") = split.next() {
        } else {
            return Err(value::Error::custom("missing elizabeth prefix"));
        }

        let topic_type = split
            .next()
            .ok_or(value::Error::custom("missing topic type"))?;
        let topic_type = TopicType::from_str(topic_type)?;

        let room = split.next().ok_or(value::Error::custom("missing room"))?;
        let room = Room::from_str(room)?;

        let device_type = split
            .next()
            .ok_or(value::Error::custom("missing device type"))?;
        let device_type = DeviceType::from_str(device_type)?;

        let capability = split
            .next()
            .ok_or(value::Error::custom("missing capability"))?;
        let capability = Capability::from_str(capability)?;

        Ok(Self {
            topic_type,
            room,
            device_type,
            capability,
        })
    }
}

impl From<&StatePayload> for Topic {
    fn from(payload: &StatePayload) -> Self {
        Self {
            topic_type: TopicType::State,
            room: payload.room,
            device_type: payload.device_type,
            capability: payload.capability,
        }
    }
}

pub fn set_topics_and_qos() -> ([String; 10], [i32; 10]) {
    (
        [
            Topic::set(LivingRoom, Recuperator, IsEnabled).to_string(),
            Topic::set(LivingRoom, Recuperator, FanSpeed).to_string(),
            Topic::set(Bedroom, Thermostat, IsEnabled).to_string(),
            Topic::set(Bedroom, Thermostat, Temperature).to_string(),
            Topic::set(HomeOffice, Thermostat, IsEnabled).to_string(),
            Topic::set(HomeOffice, Thermostat, Temperature).to_string(),
            Topic::set(LivingRoom, Thermostat, IsEnabled).to_string(),
            Topic::set(LivingRoom, Thermostat, Temperature).to_string(),
            Topic::set(Nursery, Thermostat, IsEnabled).to_string(),
            Topic::set(Nursery, Thermostat, Temperature).to_string(),
        ],
        [QOS_1; 10],
    )
}
