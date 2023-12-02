use crate::Room;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub room: Room,
    pub property: Property,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Property {
    Temperature(f32),
    Humidity(f32),
    Battery(u8),
    TemperatureAndHumidity(f32, f32),
}
