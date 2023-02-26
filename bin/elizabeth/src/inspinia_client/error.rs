use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnsupportedDevice(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnsupportedDevice(id) => f.write_fmt(format_args!("unsupported device {}", id)),
        }
    }
}

impl std::error::Error for Error {}
