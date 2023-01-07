use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use serde::Serialize;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Serialize)]
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

impl Display for UnknownPortName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("Unknown port name {}", self.0))
    }
}

impl Error for UnknownPortName {}

impl FromSql for PortName {
    fn column_result<'a>(value: ValueRef<'a>) -> std::result::Result<Self, FromSqlError> {
        PortName::try_from(value.as_str()?).map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

impl<'a> TryFrom<&'a str> for PortName {
    type Error = UnknownPortName;

    fn try_from(value: &'a str) -> std::result::Result<Self, UnknownPortName> {
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
