use crate::PropertyType;

use serde::ser::SerializeStruct;
use serde::Serialize;

#[derive(Debug, PartialEq)]
pub enum Property {
    Humidity {
        unit: HumidityUnit,
        retrievable: bool,
        reportable: bool,
    },
    Temperature {
        unit: TemperatureUnit,
        retrievable: bool,
        reportable: bool,
    },
    BatteryLevel {
        unit: BatteryLevelUnit,
        retrievable: bool,
        reportable: bool,
    },
}

impl Property {
    pub fn humidity() -> Property {
        Property::Humidity {
            unit: HumidityUnit::Percent,
            retrievable: false,
            reportable: false,
        }
    }

    pub fn temperature() -> Property {
        Property::Temperature {
            unit: TemperatureUnit::Celsius,
            retrievable: false,
            reportable: false,
        }
    }

    pub fn battery_level() -> Property {
        Property::BatteryLevel {
            unit: BatteryLevelUnit::Percent,
            retrievable: false,
            reportable: false,
        }
    }

    pub fn retrievable(self) -> Property {
        let mut value = self;

        match value {
            Property::Humidity {
                unit: _,
                ref mut retrievable,
                reportable: _,
            } => *retrievable = true,
            Property::Temperature {
                unit: _,
                ref mut retrievable,
                reportable: _,
            } => *retrievable = true,
            Property::BatteryLevel {
                unit: _,
                ref mut retrievable,
                reportable: _,
            } => *retrievable = true,
        }

        value
    }

    pub fn reportable(self) -> Property {
        let mut value = self;

        match value {
            Property::Humidity {
                unit: _,
                retrievable: _,
                ref mut reportable,
            } => *reportable = true,
            Property::Temperature {
                unit: _,
                retrievable: _,
                ref mut reportable,
            } => *reportable = true,
            Property::BatteryLevel {
                unit: _,
                retrievable: _,
                ref mut reportable,
            } => *reportable = true,
        }

        value
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum HumidityUnit {
    #[serde(rename = "unit.percent")]
    Percent,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum TemperatureUnit {
    #[serde(rename = "unit.temperature.celsius")]
    Celsius,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum BatteryLevelUnit {
    #[serde(rename = "unit.percent")]
    Percent,
}

impl serde::ser::Serialize for Property {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Parameters<U> {
            instance: &'static str,
            unit: U,
        }

        let mut property = serializer.serialize_struct("Property", 4)?;
        property.serialize_field("type", &PropertyType::Float)?;

        match self {
            Property::Humidity {
                unit,
                retrievable,
                reportable,
            } => {
                property.serialize_field("retrievable", &retrievable)?;
                property.serialize_field("reportable", &reportable)?;
                property.serialize_field(
                    "parameters",
                    &Parameters {
                        instance: "humidity",
                        unit,
                    },
                )?;
            }
            Property::Temperature {
                unit,
                retrievable,
                reportable,
            } => {
                property.serialize_field("retrievable", &retrievable)?;
                property.serialize_field("reportable", &reportable)?;
                property.serialize_field(
                    "parameters",
                    &Parameters {
                        instance: "temperature",
                        unit,
                    },
                )?;
            }
            Property::BatteryLevel {
                unit,
                retrievable,
                reportable,
            } => {
                property.serialize_field("retrievable", &retrievable)?;
                property.serialize_field("reportable", &reportable)?;
                property.serialize_field(
                    "parameters",
                    &Parameters {
                        instance: "battery_level",
                        unit,
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
            to_value(&Property::Humidity {
                unit: HumidityUnit::Percent,
                retrievable: true,
                reportable: false,
            })
            .unwrap(),
            json!({
                "type": "devices.properties.float",
                "retrievable": true,
                "reportable": false,
                "parameters": {
                    "instance": "humidity",
                    "unit": "unit.percent"
                }
            })
        );

        assert_eq!(
            to_value(&Property::Temperature {
                unit: TemperatureUnit::Celsius,
                retrievable: true,
                reportable: true,
            })
            .unwrap(),
            json!({
                "type": "devices.properties.float",
                "retrievable": true,
                "reportable": true,
                "parameters": {
                    "instance": "temperature",
                    "unit": "unit.temperature.celsius"
                }
            })
        );

        assert_eq!(
            to_value(&Property::BatteryLevel {
                unit: BatteryLevelUnit::Percent,
                retrievable: true,
                reportable: true,
            })
            .unwrap(),
            json!({
                "type": "devices.properties.float",
                "retrievable": true,
                "reportable": true,
                "parameters": {
                    "instance": "battery_level",
                    "unit": "unit.percent"
                }
            })
        );
    }
}
