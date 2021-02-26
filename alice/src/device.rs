mod capability;
mod property;

pub use capability::Capability;
pub use property::{HumidityUnit, Property, TemperatureUnit};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub description: String,
    pub room: String,

    #[serde(rename = "type")]
    pub device_type: DeviceType,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<Property>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum DeviceType {
    #[serde(rename = "devices.types.sensor")]
    Sensor,
    #[serde(rename = "devices.types.vacuum_cleaner")]
    VacuumCleaner,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{from_value, json, to_value};

    #[test]
    fn test_device_types() {
        assert_eq!(
            to_value(&DeviceType::Sensor).unwrap(),
            json!("devices.types.sensor")
        );
        assert_eq!(
            to_value(&DeviceType::VacuumCleaner).unwrap(),
            json!("devices.types.vacuum_cleaner")
        );

        assert_eq!(
            from_value::<DeviceType>(json!("devices.types.sensor")).unwrap(),
            DeviceType::Sensor
        );
        assert_eq!(
            from_value::<DeviceType>(json!("devices.types.vacuum_cleaner")).unwrap(),
            DeviceType::VacuumCleaner
        );
    }

    #[test]
    fn test_device() {
        assert_eq!(
            to_value(&Device {
                id: "test".to_string(),
                name: "Test Device".to_string(),
                description: "Test Description".to_string(),
                room: "Test Room".to_string(),
                device_type: DeviceType::Sensor,
                properties: vec![Property::humidity().retrievable()],
                capabilities: vec![],
            })
            .unwrap(),
            json!({
                "id": "test",
                "name": "Test Device",
                "description": "Test Description",
                "room": "Test Room",
                "type": "devices.types.sensor",
                "properties": [{
                    "type": "devices.properties.float",
                    "reportable": false,
                    "retrievable": true,
                    "parameters": {
                        "instance": "humidity",
                        "unit": "unit.percent"
                    }
                }]
            })
        );
    }
}
