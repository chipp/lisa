use std::fmt;

use crate::client::error::Error as ClientError;

#[derive(Debug)]
pub enum Error {
    Client(ClientError),
    Inspinia(inspinia::Error),
    Json(serde_json::Error),
    Mqtt(paho_mqtt::Error),
    Timeout(tokio::time::error::Elapsed),
    Join(tokio::task::JoinError),
}

impl From<ClientError> for Error {
    fn from(err: ClientError) -> Self {
        Self::Client(err)
    }
}

impl From<inspinia::Error> for Error {
    fn from(err: inspinia::Error) -> Self {
        Self::Inspinia(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<paho_mqtt::Error> for Error {
    fn from(err: paho_mqtt::Error) -> Self {
        Self::Mqtt(err)
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(err)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Join(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Client(err) => write!(f, "client error: {err}"),
            Self::Inspinia(err) => write!(f, "inspinia error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Mqtt(err) => write!(f, "mqtt error: {err}"),
            Self::Timeout(err) => write!(f, "timeout error: {err}"),
            Self::Join(err) => write!(f, "join error: {err}"),
        }
    }
}

impl std::error::Error for Error {}
