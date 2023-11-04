use std::fmt;

use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize,
};
use serde_json::Value;

pub enum Response {
    Ok { id: u16, result: Value },
    Err { id: u16, error: DeviceError },
}

impl Response {
    pub fn id(&self) -> u16 {
        match self {
            Response::Ok { id, result: _ } => *id,
            Response::Err { id, error: _ } => *id,
        }
    }
}

impl<'de> de::Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ResponseVisitor)
    }
}

struct ResponseVisitor;

impl<'de> Visitor<'de> for ResponseVisitor {
    type Value = Response;

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut id = None;
        let mut error = None;
        let mut result = None;

        while let Ok(Some(key)) = map.next_key() {
            match key {
                "id" => {
                    let value: u16 = map.next_value().map_err(de::Error::custom)?;
                    id = Some(value)
                }
                "error" => {
                    let value: DeviceError = map.next_value().map_err(de::Error::custom)?;
                    error = Some(value)
                }
                "result" => {
                    let value: Value = map.next_value().map_err(de::Error::custom)?;
                    result = Some(value)
                }
                _ => (),
            }
        }

        match id {
            Some(id) => match (result, error) {
                (Some(result), None) => Ok(Response::Ok { id, result }),
                (None, Some(error)) => Ok(Response::Err { id, error }),
                (None, None) => Err(de::Error::missing_field("result")),
                (Some(result), Some(error)) => {
                    panic!("got both result and error {:?} {:?}", result, error)
                }
            },
            None => Err(de::Error::missing_field("id")),
        }
    }

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "response with result or error")
    }
}

#[derive(Debug, Deserialize)]
pub struct DeviceError {
    pub code: i16,
    pub message: String,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DeviceError {}
