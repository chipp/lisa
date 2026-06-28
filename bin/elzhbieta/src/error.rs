use std::fmt;

#[derive(Debug)]
pub enum Error {
    Tuya(tuya::Error),
    Mqtt(paho_mqtt::Error),
    Json(serde_json::Error),
    Join(tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tuya(err) => write!(f, "tuya error: {err}"),
            Self::Mqtt(err) => write!(f, "mqtt error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Join(err) => write!(f, "join error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<tuya::Error> for Error {
    fn from(value: tuya::Error) -> Self {
        Self::Tuya(value)
    }
}

impl From<paho_mqtt::Error> for Error {
    fn from(value: paho_mqtt::Error) -> Self {
        Self::Mqtt(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Join(value)
    }
}
