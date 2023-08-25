use super::{Capability, DeviceType, Room};

use serde_json::Value;

pub struct UpdatePayload {
    pub device_type: DeviceType,
    pub room: Room,
    pub capability: Capability,
    pub value: Value,
}
