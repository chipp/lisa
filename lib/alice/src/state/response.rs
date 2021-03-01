use super::{Capability, Property};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,

    #[serde(rename = "ts", skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,

    payload: ResponsePayload,
}

impl Response {
    pub fn new(request_id: String, devices: Vec<ResponseDevice>) -> Response {
        Response {
            request_id: Some(request_id),
            timestamp: None,
            payload: ResponsePayload {
                user: None,
                devices,
            },
        }
    }

    pub fn notification_body(
        timestamp: i64,
        user: &'static str,
        devices: Vec<ResponseDevice>,
    ) -> Response {
        Response {
            request_id: None,
            timestamp: Some(timestamp),
            payload: ResponsePayload {
                user: Some(user),
                devices,
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct ResponsePayload {
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    user: Option<&'static str>,

    devices: Vec<ResponseDevice>,
}

#[derive(Debug, Serialize)]
pub struct ResponseDevice {
    id: String,
    properties: Vec<Property>,
    capabilities: Vec<Capability>,
}

impl ResponseDevice {
    pub fn new_with_properties(id: String, properties: Vec<Property>) -> ResponseDevice {
        ResponseDevice {
            id,
            properties,
            capabilities: vec![],
        }
    }

    pub fn new_with_capabilities(id: String, capabilities: Vec<Capability>) -> ResponseDevice {
        ResponseDevice {
            id,
            capabilities,
            properties: vec![],
        }
    }

    pub fn new_with_properties_and_capabilities(
        id: String,
        properties: Vec<Property>,
        capabilities: Vec<Capability>,
    ) -> ResponseDevice {
        ResponseDevice {
            id,
            properties,
            capabilities,
        }
    }
}
