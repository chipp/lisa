use serde::{de::value, de::IntoDeserializer, Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RangeFunction {
    Temperature,
}

impl std::str::FromStr for RangeFunction {
    type Err = value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Range {
    pub min: f32,
    pub max: f32,
    pub precision: f32,
}
