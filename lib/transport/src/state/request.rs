use crate::DeviceId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Request {
    pub device_ids: Vec<DeviceId>,
}

#[cfg(test)]
mod tests {
    use crate::Room;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialization() {
        let json = json!({
            "device_ids": [
                "recuperator/bathroom",
                "temperature_sensor/bathroom",
                "thermostat/bathroom",
                "vacuum_cleaner/bathroom",
            ],
        });

        let request = Request {
            device_ids: vec![
                DeviceId::recuperator_at_room(Room::Bathroom),
                DeviceId::temperature_sensor_at_room(Room::Bathroom),
                DeviceId::thermostat_at_room(Room::Bathroom),
                DeviceId::vacuum_cleaner_at_room(Room::Bathroom),
            ],
        };

        assert_eq!(serde_json::to_value(&request).unwrap(), json);
    }

    #[test]
    fn test_deserialization() {
        let json = json!({
            "device_ids": [
                "recuperator/bathroom",
                "temperature_sensor/bathroom",
                "thermostat/bathroom",
                "vacuum_cleaner/bathroom",
            ],
        });

        let request: Request = serde_json::from_value(json).unwrap();
        assert_eq!(
            request,
            Request {
                device_ids: vec![
                    DeviceId::recuperator_at_room(Room::Bathroom),
                    DeviceId::temperature_sensor_at_room(Room::Bathroom),
                    DeviceId::thermostat_at_room(Room::Bathroom),
                    DeviceId::vacuum_cleaner_at_room(Room::Bathroom),
                ],
            }
        );
    }
}
