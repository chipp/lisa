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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dock_error_status: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dust_collection_status: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_dust_collection: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_box_status: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_box_mode: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mop_mode: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wash_status: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wash_phase: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_shortage_status: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean_area: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean_percent: Option<i64>,
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
