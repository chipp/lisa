use std::fmt;
use std::str::FromStr;

use serde::{
    de::{value, IntoDeserializer},
    Deserialize, Serialize,
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Start,
    Stop,
    FanSpeed,
    Pause,
    Resume,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

impl FromStr for Capability {
    type Err = value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}
