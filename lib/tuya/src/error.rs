use std::fmt;
use std::net::AddrParseError;

#[derive(Debug)]
pub enum Error {
    UnsupportedVersion(String),
    InvalidLocalKey,
    InvalidPacket,
    MissingDps(&'static str),
    UnknownMode(String),
    UnresolvedDevices(Vec<String>),
    Io(std::io::Error),
    Timeout,
    Json(serde_json::Error),
    AddrParse(AddrParseError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported tuya protocol version {version}")
            }
            Self::InvalidLocalKey => f.write_str("local key must be exactly 16 bytes"),
            Self::InvalidPacket => f.write_str("invalid tuya packet"),
            Self::MissingDps(name) => write!(f, "missing dps {name}"),
            Self::UnknownMode(mode) => write!(f, "unknown tuya ac mode {mode}"),
            Self::UnresolvedDevices(devices) => {
                write!(f, "unresolved tuya devices: {}", devices.join(", "))
            }
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Timeout => f.write_str("tuya request timed out"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::AddrParse(err) => write!(f, "address parse error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<AddrParseError> for Error {
    fn from(value: AddrParseError) -> Self {
        Self::AddrParse(value)
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::Timeout
    }
}
