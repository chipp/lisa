use super::port_name::PortName;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    #[serde(with = "parse_controls")]
    pub controls: Vec<PortName>,
    pub min_temp: u8,
    pub max_temp: u8,
    pub step: f32,
}

mod parse_controls {
    use super::super::port_name::{PortName, ALL_PORT_NAMES};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<PortName>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let mut names = vec![];

        for name in string.split(",") {
            names.push(
                PortName::try_from(name)
                    .map_err(|_| serde::de::Error::unknown_variant(name, &ALL_PORT_NAMES))?,
            );
        }

        Ok(names)
    }
}
