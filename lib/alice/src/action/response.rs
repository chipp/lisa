mod capability;
pub use capability::{ActionResult, Capability, Error, ErrorCode};

use serde::Serialize;
use transport::DeviceId;

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
    id: DeviceId,
    capabilities: Vec<Capability>,
}

impl ResponseDevice {
    pub fn new(id: DeviceId, capabilities: Vec<Capability>) -> ResponseDevice {
        ResponseDevice { id, capabilities }
    }

    pub fn capabilities_mut(&mut self) -> &mut Vec<Capability> {
        &mut self.capabilities
    }
}
