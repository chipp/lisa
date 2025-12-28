use std::fmt;

use serde::{de::value, de::IntoDeserializer, Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModeFunction {
    WorkSpeed,
    FanSpeed,
    CleanupMode,
}

impl std::str::FromStr for ModeFunction {
    type Err = value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Mode {
    Quiet,
    Low,
    #[default]
    Normal,
    Medium,
    High,
    Turbo,
    DryCleaning,
    WetCleaning,
    MixedCleaning,
}

impl std::str::FromStr for Mode {
    type Err = value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}
