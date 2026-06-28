use serde::{Deserialize, Serialize};

use crate::Room;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Action {
    pub room: Room,
    pub action_type: ActionType,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    SetIsEnabled(bool),
    SetMode(Mode),
    SetTargetTemperature(f32),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub room: Room,
    pub is_enabled: bool,
    pub mode: Mode,
    pub target_temperature: f32,
    pub current_temperature: f32,
    pub humidity: u8,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Cool,
    Heat,
    Dry,
    FanOnly,
    Auto,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_action_serialization() {
        assert_eq!(
            serde_json::to_value(Action {
                room: Room::Bedroom,
                action_type: ActionType::SetIsEnabled(true),
            })
            .unwrap(),
            json!({"room": "bedroom", "action_type": {"set_is_enabled": true}})
        );
        assert_eq!(
            serde_json::to_value(Action {
                room: Room::Bedroom,
                action_type: ActionType::SetMode(Mode::Cool),
            })
            .unwrap(),
            json!({"room": "bedroom", "action_type": {"set_mode": "cool"}})
        );
        assert_eq!(
            serde_json::to_value(Action {
                room: Room::Bedroom,
                action_type: ActionType::SetTargetTemperature(22.5),
            })
            .unwrap(),
            json!({"room": "bedroom", "action_type": {"set_target_temperature": 22.5}})
        );
    }

    #[test]
    fn test_state_serialization() {
        assert_eq!(
            serde_json::to_value(State {
                room: Room::Bedroom,
                is_enabled: true,
                mode: Mode::Cool,
                target_temperature: 22.0,
                current_temperature: 27.0,
                humidity: 45,
            })
            .unwrap(),
            json!({
                "room": "bedroom",
                "is_enabled": true,
                "mode": "cool",
                "target_temperature": 22.0,
                "current_temperature": 27.0,
                "humidity": 45
            })
        );
    }
}
