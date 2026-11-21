use transport::DeviceId;

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

#[derive(Debug, Clone, Serialize)]
pub struct ResponseDevice {
    id: DeviceId,
    properties: Vec<Property>,
    capabilities: Vec<Capability>,
}

impl ResponseDevice {
    pub fn id(&self) -> DeviceId {
        self.id
    }

    /// Apply newer fields while retaining fields absent from the update.
    pub fn merge(&mut self, newer: ResponseDevice) -> usize {
        assert_eq!(self.id, newer.id);
        let mut replaced = 0;
        for property in newer.properties {
            let before = self.properties.len();
            self.properties
                .retain(|old| std::mem::discriminant(old) != std::mem::discriminant(&property));
            replaced += before - self.properties.len();
            self.properties.push(property);
        }
        for capability in newer.capabilities {
            let before = self.capabilities.len();
            self.capabilities
                .retain(|old| !same_capability(old, &capability));
            replaced += before - self.capabilities.len();
            self.capabilities.push(capability);
        }
        replaced
    }

    pub fn field_count(&self) -> usize {
        self.properties.len() + self.capabilities.len()
    }
}

fn same_capability(a: &Capability, b: &Capability) -> bool {
    match (a, b) {
        (Capability::OnOff { .. }, Capability::OnOff { .. }) => true,
        (Capability::Mode { function: a, .. }, Capability::Mode { function: b, .. }) => a == b,
        (Capability::Toggle { function: a, .. }, Capability::Toggle { function: b, .. }) => a == b,
        (Capability::Range { function: a, .. }, Capability::Range { function: b, .. }) => a == b,
        _ => false,
    }
}

impl ResponseDevice {
    pub fn new_empty(id: DeviceId) -> ResponseDevice {
        ResponseDevice {
            id,
            properties: vec![],
            capabilities: vec![],
        }
    }

    pub fn new_with_properties(id: DeviceId, properties: Vec<Property>) -> ResponseDevice {
        ResponseDevice {
            id,
            properties,
            capabilities: vec![],
        }
    }

    pub fn new_with_capabilities(id: DeviceId, capabilities: Vec<Capability>) -> ResponseDevice {
        ResponseDevice {
            id,
            capabilities,
            properties: vec![],
        }
    }

    pub fn new_with_properties_and_capabilities(
        id: DeviceId,
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
