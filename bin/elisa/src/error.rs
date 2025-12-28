use std::fmt;

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    Mqtt(paho_mqtt::Error),
    Vacuum(roborock::Error),
    QueueClosed,
    Join(tokio::task::JoinError),
    AddrParse(std::net::AddrParseError),
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

impl From<roborock::Error> for Error {
    fn from(err: roborock::Error) -> Self {
        Self::Vacuum(err)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Join(err)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(err: std::net::AddrParseError) -> Self {
        Self::AddrParse(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Mqtt(err) => write!(f, "mqtt error: {err}"),
            Self::Vacuum(err) => write!(f, "vacuum error: {err}"),
            Self::QueueClosed => write!(f, "vacuum queue closed"),
            Self::Join(err) => write!(f, "join error: {err}"),
            Self::AddrParse(err) => write!(f, "address parse error: {err}"),
        }
    }
}

impl std::error::Error for Error {}
