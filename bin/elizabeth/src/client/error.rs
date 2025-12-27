use std::fmt;

use transport::{DeviceType, Room};

#[derive(Debug)]
pub enum Error {
    UnsupportedDevice(String),
    MissingCapability(&'static str, DeviceType, Room),
    MissingPort(&'static str, DeviceType, Room),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnsupportedDevice(id) => f.write_fmt(format_args!("unsupported device {}", id)),
            Error::MissingCapability(capability, device_type, room) => f.write_fmt(format_args!(
                "missing capability `{}` for device {} in room {}",
                capability, device_type, room
            )),
            Error::MissingPort(port, device_type, room) => f.write_fmt(format_args!(
                "missing port `{}` for device {} in room {}",
                port, device_type, room
            )),
        }
    }
}

impl std::error::Error for Error {}
