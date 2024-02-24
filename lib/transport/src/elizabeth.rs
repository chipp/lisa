use serde::{Deserialize, Serialize};

use crate::{DeviceType, Room};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Action {
    pub room: Room,
    pub device_type: DeviceType,
    pub action_type: ActionType,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    SetIsEnabled(bool),
    SetFanSpeed(FanSpeed),
    SetTemperature(f32, bool),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct State {
    pub room: Room,
    pub device_type: DeviceType,
    pub capability: Capability,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CurrentState {
    pub room: Room,
    pub device_type: DeviceType,
    pub capabilities: Vec<Capability>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    IsEnabled(bool),
    FanSpeed(FanSpeed),
    CurrentTemperature(f32),
    Temperature(f32),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FanSpeed {
    Low,
    Medium,
    High,
}
