use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::PropertyType;

#[derive(Debug)]
pub enum Property {
    Humidity { value: f32 },
    Temperature { value: f32 },
}

impl Property {
    pub fn humidity(value: f32) -> Property {
        Property::Humidity { value }
    }

    pub fn temperature(value: f32) -> Property {
        Property::Temperature { value }
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

        let mut property = serializer.serialize_struct("Property", 4)?;

        match self {
            Property::Humidity { value } => {
                property.serialize_field("type", &PropertyType::Float)?;
                property.serialize_field(
                    "state",
                    &State {
                        instance: "humidity",
                        value,
                    },
                )?;
            }
            Property::Temperature { value } => {
                property.serialize_field("type", &PropertyType::Float)?;
                property.serialize_field(
                    "state",
                    &State {
                        instance: "temperature",
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
    }
}
