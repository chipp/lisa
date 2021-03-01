use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum PropertyType {
    #[serde(rename = "devices.properties.float")]
    Float,
}
