use std::{fmt, str::FromStr};

use serde::{
    de::{value, IntoDeserializer},
    Deserialize, Serialize,
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    IsEnabled,
    FanSpeed,
    Temperature,
    CurrentTemperature,
    Mode,
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
