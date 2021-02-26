use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "instance", rename_all = "snake_case")]
pub enum Parameters {
    Humidity { unit: HumidityUnit },
    Temperature { unit: TemperatureUnit },
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
    #[serde(rename = "unit.temperature.kelvin")]
    Kelvin,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn test_parameters() {
        assert_eq!(
            to_value(&Parameters::Humidity {
                unit: HumidityUnit::Percent
            })
            .unwrap(),
            json!({"instance": "humidity", "unit": "unit.percent"})
        );

        assert_eq!(
            to_value(&Parameters::Temperature {
                unit: TemperatureUnit::Celsius
            })
            .unwrap(),
            json!({"instance": "temperature", "unit": "unit.temperature.celsius"})
        );

        assert_eq!(
            to_value(&Parameters::Temperature {
                unit: TemperatureUnit::Kelvin
            })
            .unwrap(),
            json!({"instance": "temperature", "unit": "unit.temperature.kelvin"})
        );
    }
}
