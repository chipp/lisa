use crate::StateCapability;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Request<'a> {
    #[serde(borrow = "'a")]
    pub payload: Payload<'a>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Payload<'a> {
    #[serde(borrow = "'a")]
    pub devices: Vec<RequestDevice<'a>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RequestDevice<'a> {
    pub id: &'a str,
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
                    "id": "socket-001-xda",
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
        assert_eq!(devices[0].id, "socket-001-xda");
        assert_eq!(devices[0].capabilities.len(), 1);
        assert_eq!(devices[0].capabilities[0], StateCapability::on_off(true));
    }
}
