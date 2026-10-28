use crate::Room;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Action {
    pub room: Room,
    pub action_type: ActionType,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    SetIsEnabled(bool),
    SetFanSpeed(FanSpeed),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FanSpeed {
    Low,
    Medium,
    High,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct State {
    pub room: Room,
    pub is_enabled: bool,
    pub fan_speed: FanSpeed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        action::Action as RequestAction,
        state::{StateResponse, StateUpdate},
    };
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn mqtt_payloads_round_trip() {
        let action = RequestAction::Elizaveta(
            Action {
                room: Room::LivingRoom,
                action_type: ActionType::SetFanSpeed(FanSpeed::Medium),
            },
            Uuid::nil(),
        );
        let value = serde_json::to_value(&action).unwrap();
        assert_eq!(
            value,
            json!({"elizaveta": [{"room":"living_room", "action_type":{"set_fan_speed":"medium"}}, "00000000-0000-0000-0000-000000000000"]})
        );
        assert_eq!(
            serde_json::from_value::<RequestAction>(value).unwrap(),
            action
        );
        let state = State {
            room: Room::LivingRoom,
            is_enabled: true,
            fan_speed: FanSpeed::High,
        };
        for value in [
            serde_json::to_value(StateResponse::Elizaveta(state)).unwrap(),
            serde_json::to_value(StateUpdate::Elizaveta(state)).unwrap(),
        ] {
            assert_eq!(
                value,
                json!({"elizaveta": {"room":"living_room", "is_enabled":true, "fan_speed":"high"}})
            );
        }
    }
}
