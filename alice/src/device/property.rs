use super::parameters::Parameters;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub property_type: PropertyType,
    pub retrievable: bool,
    pub reportable: bool,
    pub parameters: Parameters,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum PropertyType {
    #[serde(rename = "devices.properties.float")]
    Float,
}

#[cfg(test)]
mod tests {
    use super::super::parameters::HumidityUnit;
    use super::*;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn test_property_types() {
        assert_eq!(
            to_value(&PropertyType::Float).unwrap(),
            json!("devices.properties.float")
        );

        assert_eq!(
            from_value::<PropertyType>(json!("devices.properties.float")).unwrap(),
            PropertyType::Float
        );
    }

    #[test]
    fn test_property() {
        assert_eq!(
            to_value(&Property {
                property_type: PropertyType::Float,
                retrievable: true,
                reportable: false,
                parameters: Parameters::Humidity {
                    unit: HumidityUnit::Percent,
                },
            })
            .unwrap(),
            json!({
                "type": "devices.properties.float",
                "reportable": false,
                "retrievable": true,
                "parameters": {
                    "instance": "humidity",
                    "unit": "unit.percent"
                }
            })
        );
    }
}
