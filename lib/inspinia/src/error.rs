use std::fmt;

use crate::device_manager::error::Error as DeviceManagerError;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub enum Error {
    StreamClosed,
    UnexpectedMessage(Message),
    Pong,
    WebSocketError(tokio_tungstenite::tungstenite::error::Error),
    DeviceManager(DeviceManagerError),
    Http(chipp_http::Error),
    Io(std::io::Error),
    Rusqlite(rusqlite::Error),
    SerdeJson(serde_json::Error),
    Zip(zip::result::ZipError),
}

impl From<DeviceManagerError> for Error {
    fn from(err: DeviceManagerError) -> Self {
        Self::DeviceManager(err)
    }
}

impl From<chipp_http::Error> for Error {
    fn from(err: chipp_http::Error) -> Self {
        Self::Http(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::Rusqlite(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}

impl From<tokio_tungstenite::tungstenite::error::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::error::Error) -> Self {
        Self::WebSocketError(err)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        Self::Zip(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StreamClosed => write!(f, "stream closed"),
            Self::UnexpectedMessage(message) => write!(f, "unexpected message: {:?}", message),
            Self::Pong => write!(f, "pong"),
            Self::WebSocketError(err) => write!(f, "websocket error: {err}"),
            Self::DeviceManager(err) => write!(f, "device manager error: {err}"),
            Self::Http(err) => write!(f, "http error: {err}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Rusqlite(err) => write!(f, "rusqlite error: {err}"),
            Self::SerdeJson(err) => write!(f, "json error: {err}"),
            Self::Zip(err) => write!(f, "zip error: {err}"),
        }
    }
}

impl std::error::Error for Error {}
