use std::fmt;

#[derive(Debug)]
pub enum Error {
    Mqtt(paho_mqtt::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    Join(tokio::task::JoinError),
    Http(chipp_http::Error),
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Join(err)
    }
}

impl From<chipp_http::Error> for Error {
    fn from(err: chipp_http::Error) -> Self {
        Self::Http(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mqtt(err) => write!(f, "mqtt error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Join(err) => write!(f, "join error: {err}"),
            Self::Http(err) => write!(f, "http error: {err}"),
        }
    }
}

impl std::error::Error for Error {}
