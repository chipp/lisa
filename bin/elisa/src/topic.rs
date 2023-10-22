use std::{fmt, str::FromStr};

use crate::capability::Capability::{self, *};
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
    pub topic_type: TopicType,
    pub capability: Capability,
}

impl Topic {
    pub const fn state(capability: Capability) -> Self {
        Self {
            topic_type: TopicType::State,
            capability,
        }
    }

    pub const fn set(capability: Capability) -> Self {
        Self {
            topic_type: TopicType::Set,
            capability,
        }
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "elisa/{}/{}/{}",
            self.topic_type, "vacuum", self.capability
        )
    }
}

impl FromStr for Topic {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Topic, value::Error> {
        let mut split = s.split('/');

        if let Some("elisa") = split.next() {
        } else {
            return Err(value::Error::custom("missing elisa prefix"));
        }

        let topic_type = split
            .next()
            .ok_or(value::Error::custom("missing topic type"))?;
        let topic_type = TopicType::from_str(topic_type)?;

        if let Some("vacuum") = split.next() {
        } else {
            return Err(value::Error::custom("missing device type"));
        }

        let capability = split
            .next()
            .ok_or(value::Error::custom("missing capability"))?;
        let capability = Capability::from_str(capability)?;

        Ok(Self {
            topic_type,
            capability,
        })
    }
}

pub fn set_topics_and_qos() -> ([String; 5], [i32; 10]) {
    (
        [
            Topic::set(Start).to_string(),
            Topic::set(Stop).to_string(),
            Topic::set(FanSpeed).to_string(),
            Topic::set(Pause).to_string(),
            Topic::set(Resume).to_string(),
        ],
        [QOS_1; 10],
    )
}
