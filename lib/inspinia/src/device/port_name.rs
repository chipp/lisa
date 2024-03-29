use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use serde::Serialize;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortName {
    OnOff,
    SetTemp,
    FanSpeed,
    RoomTemp,
    Mode,
}

pub const ALL_PORT_NAMES: [&str; 5] = ["ON_OFF", "SET_TEMP", "FAN_SPEED", "ROOM_TEMP", "MODE"];

#[derive(Debug)]
pub struct UnknownPortName(String);

impl fmt::Display for UnknownPortName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Unknown port name {}", self.0))
    }
}

impl Error for UnknownPortName {}

impl FromSql for PortName {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        PortName::from_str(value.as_str()?).map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

impl FromStr for PortName {
    type Err = UnknownPortName;

    fn from_str(value: &str) -> Result<Self, UnknownPortName> {
        match value {
            "ON_OFF" => Ok(Self::OnOff),
            "SET_TEMP" => Ok(Self::SetTemp),
            "FAN_SPEED" => Ok(Self::FanSpeed),
            "ROOM_TEMP" => Ok(Self::RoomTemp),
            "MODE" => Ok(Self::Mode),
            _ => Err(UnknownPortName(value.to_string())),
        }
    }
}
