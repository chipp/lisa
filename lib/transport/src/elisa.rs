use serde::{Deserialize, Serialize};

use crate::Room;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Start(Vec<Room>),
    Stop,
    SetWorkSpeed(WorkSpeed),
    Pause,
    Resume,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub battery_level: u8,
    pub is_enabled: bool,
    pub is_paused: bool,
    pub work_speed: WorkSpeed,
    pub rooms: Vec<Room>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkSpeed {
    Silent,
    Standard,
    Medium,
    Turbo,
}
