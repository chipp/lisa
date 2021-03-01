use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::PropertyType;

#[derive(Debug)]
pub enum Property {
    Humidity { value: f32 },
    Temperature { value: f32 },
    BatteryLevel { value: f32 },
}

impl Property {
    pub fn humidity(value: f32) -> Property {
        Property::Humidity { value }
    }

    pub fn temperature(value: f32) -> Property {
        Property::Temperature { value }
    }

    pub fn battery_level(value: f32) -> Property {
        Property::BatteryLevel { value }
    }
}

impl serde::ser::Serialize for Property {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct State<U> {
            instance: &'static str,
            value: U,
        }

        let mut property = serializer.serialize_struct("Property", 2)?;
        property.serialize_field("type", &PropertyType::Float)?;

        match self {
            Property::Humidity { value } => {
                property.serialize_field(
                    "state",
                    &State {
                        instance: "humidity",
                        value,
                    },
                )?;
            }
            Property::Temperature { value } => {
                property.serialize_field(
                    "state",
                    &State {
                        instance: "temperature",
                        value,
                    },
                )?;
            }
            Property::BatteryLevel { value } => {
                property.serialize_field(
                    "state",
                    &State {
                        instance: "battery_level",
                        value,
                    },
                )?;
            }
        }

        property.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn test_properties() {
        assert_eq!(
            to_value(&Property::Humidity { value: 55.5 }).unwrap(),
            json!({
                "type": "devices.properties.float",
                "state": {"instance": "humidity", "value": 55.5}
            })
        );

        assert_eq!(
            to_value(&Property::Temperature { value: 23.0 }).unwrap(),
            json!({
                "type": "devices.properties.float",
                "state": {"instance": "temperature", "value": 23.0}
            })
        );

        assert_eq!(
            to_value(&Property::BatteryLevel { value: 98.0 }).unwrap(),
            json!({
                "type": "devices.properties.float",
                "state": {"instance": "battery_level", "value": 98.0}
            })
        );
    }
}
