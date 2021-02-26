use super::{Capability, Property};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Response {
    request_id: String,
    payload: ResponsePayload,
}

impl Response {
    pub fn new(request_id: String, devices: Vec<ResponseDevice>) -> Response {
        Response {
            request_id,
            payload: ResponsePayload { devices },
        }
    }
}

#[derive(Debug, Serialize)]
struct ResponsePayload {
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
