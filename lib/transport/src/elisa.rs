use serde::{Deserialize, Serialize};

use crate::Room;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Start(Vec<Room>),
    Stop,
    SetWorkSpeed(WorkSpeed),
    SetCleanupMode(CleanupMode),
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
    pub cleanup_mode: CleanupMode,
    pub rooms: Vec<Room>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum WorkSpeed {
    Min,
    Silent,
    Standard,
    Medium,
    Turbo,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CleanupMode {
    DryCleaning,
    WetCleaning,
    MixedCleaning,
}
