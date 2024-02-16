mod capability;
mod property;

pub use capability::Capability;
pub use property::{Property, TemperatureUnit};

use serde::{Deserialize, Serialize};
use transport::DeviceId;

#[derive(Debug, Serialize)]
pub struct Device {
    pub id: DeviceId,
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
    #[serde(rename = "devices.types.thermostat")]
    Thermostat,
    #[serde(rename = "devices.types.thermostat.ac")]
    ThermostatAc,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{from_value, json, to_value};
    use transport::Room;

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
            to_value(&DeviceType::Thermostat).unwrap(),
            json!("devices.types.thermostat")
        );
        assert_eq!(
            to_value(&DeviceType::ThermostatAc).unwrap(),
            json!("devices.types.thermostat.ac")
        );

        assert_eq!(
            from_value::<DeviceType>(json!("devices.types.sensor")).unwrap(),
            DeviceType::Sensor
        );
        assert_eq!(
            from_value::<DeviceType>(json!("devices.types.vacuum_cleaner")).unwrap(),
            DeviceType::VacuumCleaner
        );
        assert_eq!(
            from_value::<DeviceType>(json!("devices.types.thermostat")).unwrap(),
            DeviceType::Thermostat
        );
        assert_eq!(
            from_value::<DeviceType>(json!("devices.types.thermostat.ac")).unwrap(),
            DeviceType::ThermostatAc
        );
    }

    #[test]
    fn test_device() {
        assert_eq!(
            to_value(Device {
                id: DeviceId::vacuum_cleaner_at_room(Room::Kitchen),
                name: "Test Device".to_string(),
                description: "Test Description".to_string(),
                room: "Test Room".to_string(),
                device_type: DeviceType::Sensor,
                properties: vec![Property::humidity().retrievable()],
                capabilities: vec![],
            })
            .unwrap(),
            json!({
                "id": "vacuum_cleaner/kitchen",
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
