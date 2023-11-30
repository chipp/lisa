use crate::StateCapability;

use serde::Deserialize;
use transport::DeviceId;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub payload: Payload,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Payload {
    pub devices: Vec<RequestDevice>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RequestDevice {
    pub id: DeviceId,
    pub capabilities: Vec<StateCapability>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_slice, json, to_vec};

    #[test]
    fn test_request_parsing() {
        let json = json!({
            "payload": {
                "devices": [{
                    "id": "thermostat/kitchen",
                    "custom_data": {
                      "api_location": "rus"
                    },
                    "capabilities": [{
                         "type": "devices.capabilities.on_off",
                         "state": {
                           "instance": "on",
                           "value": true
                         }
                    }]
                }]
            }
        });
        let json = to_vec(&json).unwrap();

        let devices = from_slice::<Request>(&json).unwrap().payload.devices;

        assert_eq!(devices.len(), 1);
        assert_eq!(
            devices[0].id,
            DeviceId::thermostat_at_room(transport::Room::Kitchen)
        );
        assert_eq!(devices[0].capabilities.len(), 1);
        assert_eq!(devices[0].capabilities[0], StateCapability::on_off(true));
    }
}
