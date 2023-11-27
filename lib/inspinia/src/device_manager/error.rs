use std::fmt;

use crate::Room;

#[derive(Debug)]
pub enum Error {
    NoThermostatInRoom(Room),
    NoRecuperatorInRoom(Room),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoThermostatInRoom(room) => {
                f.write_fmt(format_args!("No thermostat in room {:?}", room))
            }
            Self::NoRecuperatorInRoom(room) => {
                f.write_fmt(format_args!("No recuperator in room {:?}", room))
            }
        }
    }
}

impl std::error::Error for Error {}
