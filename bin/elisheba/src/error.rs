use std::fmt;

#[derive(Debug)]
pub enum Error {
    Sonoff(sonoff::Error),
    Mqtt(paho_mqtt::Error),
    Json(serde_json::Error),
    Join(tokio::task::JoinError),
    Timeout(tokio::time::error::Elapsed),
    Io(std::io::Error),
    UnknownDevice,
}

impl From<sonoff::Error> for Error {
    fn from(err: sonoff::Error) -> Self {
        Self::Sonoff(err)
    }
}

impl From<paho_mqtt::Error> for Error {
    fn from(err: paho_mqtt::Error) -> Self {
        Self::Mqtt(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Join(err)
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sonoff(err) => write!(f, "sonoff error: {err}"),
            Self::Mqtt(err) => write!(f, "mqtt error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Join(err) => write!(f, "join error: {err}"),
            Self::Timeout(err) => write!(f, "timeout error: {err}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::UnknownDevice => write!(f, "unknown device"),
        }
    }
}

impl std::error::Error for Error {}
