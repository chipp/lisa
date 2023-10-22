use std::fmt::{self, Display};
use std::str::FromStr;

use crate::action::Action::{self, *};
use crate::capability::Capability;
use paho_mqtt::QOS_1;
use serde::{
    de::{value, Error, IntoDeserializer},
    Deserialize, Serialize,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicType {
    Action,
    State,
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

trait Feature: FromStr + Display {}

#[derive(Debug)]
pub struct Topic<F> {
    pub topic_type: TopicType,
    pub feature: F,
}

impl Feature for Action {}
impl Feature for Capability {}

impl Topic<Action> {
    pub const fn action(feature: Action) -> Topic<Action> {
        Topic {
            topic_type: TopicType::Action,
            feature,
        }
    }
}

impl Topic<Capability> {
    pub const fn state(feature: Capability) -> Topic<Capability> {
        Topic {
            topic_type: TopicType::State,
            feature,
        }
    }
}

impl<F: Display> fmt::Display for Topic<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "elisa/{}/none/{}/{}",
            self.topic_type, "vacuum", self.feature
        )
    }
}

impl<F: FromStr<Err = value::Error>> FromStr for Topic<F> {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Topic<F>, Self::Err>
    where
        F::Err: serde::de::Error,
    {
        let mut split = s.split('/');

        if let Some("elisa") = split.next() {
        } else {
            return Err(value::Error::custom("missing elisa prefix"));
        }

        let topic_type = split
            .next()
            .ok_or(value::Error::custom("missing topic type"))?;
        let topic_type = TopicType::from_str(topic_type)?;

        if let Some("none") = split.next() {
        } else {
            return Err(value::Error::custom("missing room"));
        }

        if let Some("vacuum") = split.next() {
        } else {
            return Err(value::Error::custom("missing device type"));
        }

        let feature = split.next().ok_or(F::Err::custom("missing feature"))?;
        let feature = F::from_str(feature)?;

        Ok(Self {
            topic_type,
            feature,
        })
    }
}

pub fn actions_topics_and_qos() -> ([String; 5], [i32; 10]) {
    (
        [
            Topic::action(Start).to_string(),
            Topic::action(Stop).to_string(),
            Topic::action(SetFanSpeed).to_string(),
            Topic::action(Pause).to_string(),
            Topic::action(Resume).to_string(),
        ],
        [QOS_1; 10],
    )
}
