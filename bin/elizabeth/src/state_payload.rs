use std::{fmt, str::FromStr};

use crate::DeviceType;
use inspinia::{PortName, Room};

use serde::{
    de::{value, IntoDeserializer},
    Deserialize, Serialize,
};

#[derive(Debug)]
pub struct StatePayload {
    pub device_type: DeviceType,
    pub room: Room,
    pub capability: Capability,
    pub value: String,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    IsEnabled,
    FanSpeed,
    CurrentTemperature,
    Temperature,
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

impl From<PortName> for Capability {
    fn from(name: PortName) -> Self {
        match name {
            PortName::OnOff => Capability::IsEnabled,
            PortName::FanSpeed => Capability::FanSpeed,
            PortName::SetTemp => Capability::Temperature,
            PortName::RoomTemp => Capability::CurrentTemperature,
            PortName::Mode => Capability::Mode,
        }
    }
}
