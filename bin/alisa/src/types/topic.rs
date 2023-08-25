use std::{fmt, str::FromStr};

use crate::{
    types::Capability::{self, *},
    types::DeviceType::{self, *},
    types::Room::{self, *},
    types::UpdatePayload,
};

use paho_mqtt::QOS_1;
use serde::{
    de::{value, Error, IntoDeserializer},
    Deserialize, Serialize,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    Elizabeth,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

impl FromStr for Service {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicType {
    State,
    Set,
}

impl fmt::Display for TopicType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
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
    pub service: Service,
    pub topic_type: TopicType,
    pub room: Room,
    pub device_type: DeviceType,
    pub capability: Capability,
}

impl Topic {
    pub const fn state(
        service: Service,
        room: Room,
        device_type: DeviceType,
        capability: Capability,
    ) -> Self {
        Self {
            service,
            topic_type: TopicType::State,
            room,
            device_type,
            capability,
        }
    }

    pub const fn set(
        service: Service,
        room: Room,
        device_type: DeviceType,
        capability: Capability,
    ) -> Self {
        Self {
            service,
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

        let service = split
            .next()
            .ok_or(value::Error::custom("missing service"))?;
        let service = Service::from_str(service)?;

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
            service,
            topic_type,
            room,
            device_type,
            capability,
        })
    }
}

impl From<(Service, &UpdatePayload)> for Topic {
    fn from(value: (Service, &UpdatePayload)) -> Self {
        Self {
            service: value.0,
            topic_type: TopicType::Set,
            room: value.1.room,
            device_type: value.1.device_type,
            capability: value.1.capability,
        }
    }
}

pub fn state_topics_and_qos() -> ([String; 10], [i32; 10]) {
    (
        [
            Topic::state(Service::Elizabeth, LivingRoom, Recuperator, IsEnabled).to_string(),
            Topic::state(Service::Elizabeth, LivingRoom, Recuperator, FanSpeed).to_string(),
            Topic::state(Service::Elizabeth, Bedroom, Thermostat, IsEnabled).to_string(),
            Topic::state(Service::Elizabeth, Bedroom, Thermostat, Temperature).to_string(),
            Topic::state(Service::Elizabeth, HomeOffice, Thermostat, IsEnabled).to_string(),
            Topic::state(Service::Elizabeth, HomeOffice, Thermostat, Temperature).to_string(),
            Topic::state(Service::Elizabeth, LivingRoom, Thermostat, IsEnabled).to_string(),
            Topic::state(Service::Elizabeth, LivingRoom, Thermostat, Temperature).to_string(),
            Topic::state(Service::Elizabeth, Nursery, Thermostat, IsEnabled).to_string(),
            Topic::state(Service::Elizabeth, Nursery, Thermostat, Temperature).to_string(),
        ],
        [QOS_1; 10],
    )
}
