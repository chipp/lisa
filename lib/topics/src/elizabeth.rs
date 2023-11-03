use serde::{Deserialize, Serialize};
use str_derive::Str;

use crate::Feature;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Str)]
#[serde(rename_all = "snake_case")]
pub enum State {
    IsEnabled,
    FanSpeed,
    CurrentTemperature,
    Temperature,
    Mode,
}

impl Feature for State {
    fn service() -> crate::Service {
        crate::Service::Elizabeth
    }
}
