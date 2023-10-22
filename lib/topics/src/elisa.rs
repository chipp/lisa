use serde::{Deserialize, Serialize};
use str_derive::Str;

use crate::Feature;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Start,
    Stop,
    SetFanSpeed,
    Pause,
    Resume,
}

impl Feature for Action {}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Status,
}

impl Feature for State {}
