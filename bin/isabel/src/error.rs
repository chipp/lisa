use std::fmt;

#[derive(Debug)]
pub enum Error {
    Bluetooth(bluetooth::Error),
    Mqtt(paho_mqtt::Error),
    Json(serde_json::Error),
    Timeout(tokio::time::error::Elapsed),
}

impl From<bluetooth::Error> for Error {
    fn from(err: bluetooth::Error) -> Self {
        Self::Bluetooth(err)
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

impl From<tokio::time::error::Elapsed> for Error {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bluetooth(err) => write!(f, "bluetooth error: {err}"),
            Self::Mqtt(err) => write!(f, "mqtt error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Timeout(err) => write!(f, "timeout error: {err}"),
        }
    }
}

impl std::error::Error for Error {}
