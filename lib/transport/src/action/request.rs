use crate::{elisa, elizabeth};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Request {
    pub actions: Vec<Action>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Elisa(elisa::Action, uuid::Uuid),
    Elizabeth(elizabeth::Action, uuid::Uuid),
}

impl Action {
    pub fn id(&self) -> uuid::Uuid {
        match self {
            Action::Elisa(_, id) => *id,
            Action::Elizabeth(_, id) => *id,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{elizabeth::ActionType, DeviceType, Room};
    use serde_json::json;
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_action_id() {
        let id = uuid::Uuid::new_v4();
        let action = Action::Elisa(elisa::Action::Stop, id);
        assert_eq!(action.id(), id);

        let id = uuid::Uuid::new_v4();
        let action = Action::Elizabeth(
            elizabeth::Action {
                room: Room::Bathroom,
                device_type: DeviceType::Recuperator,
                action_type: ActionType::SetIsEnabled(true),
            },
            id,
        );
        assert_eq!(action.id(), id);
    }

    #[test]
    fn test_elizabeth_serialization() {
        let id = uuid!("CFF182E2-2BCB-4C19-A070-43D43EF7C104");
        let action = Action::Elizabeth(
            elizabeth::Action {
                room: Room::Bathroom,
                device_type: DeviceType::Recuperator,
                action_type: ActionType::SetIsEnabled(true),
            },
            id,
        );

        let serialized = serde_json::to_string(&action).unwrap();
        assert_eq!(
            serialized,
            r#"{"elizabeth":[{"room":"bathroom","device_type":"recuperator","action_type":{"set_is_enabled":true}},"cff182e2-2bcb-4c19-a070-43d43ef7c104"]}"#
        );
    }

    #[test]
    fn test_elisa_serialization() {
        let id = uuid!("2E363D79-5D42-4F11-955E-7B2046319943");
        let action = Action::Elisa(elisa::Action::Stop, id);

        let serialized = serde_json::to_string(&action).unwrap();
        assert_eq!(
            serialized,
            r#"{"elisa":["stop","2e363d79-5d42-4f11-955e-7b2046319943"]}"#
        );

        let id = uuid!("48FE7DE3-C3A9-47BA-A1A3-3E9C3FFC910E");
        let action = Action::Elisa(elisa::Action::Start(vec![Room::Bathroom, Room::Toilet]), id);

        let serialized = serde_json::to_string(&action).unwrap();
        assert_eq!(
            serialized,
            r#"{"elisa":[{"start":["bathroom","toilet"]},"48fe7de3-c3a9-47ba-a1a3-3e9c3ffc910e"]}"#
        );
    }

    #[test]
    fn test_deserialization() {
        let id = uuid!("2E363D79-5D42-4F11-955E-7B2046319943");
        let action = Action::Elisa(elisa::Action::Stop, id);

        let json = json!({
            "elisa": [
                "stop",
                "2e363d79-5d42-4f11-955e-7b2046319943"
            ]
        });

        let deserialized: Action = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, action);

        let id = uuid!("48FE7DE3-C3A9-47BA-A1A3-3E9C3FFC910E");
        let action = Action::Elisa(elisa::Action::Start(vec![Room::Bathroom, Room::Toilet]), id);

        let json = json!({
            "elisa": [
                {
                    "start": [
                        "bathroom",
                        "toilet"
                    ]
                },
                "48fe7de3-c3a9-47ba-a1a3-3e9c3ffc910e"
            ]
        });

        let deserialized: Action = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, action);

        let id = uuid!("CFF182E2-2BCB-4C19-A070-43D43EF7C104");
        let action = Action::Elizabeth(
            elizabeth::Action {
                room: Room::Bathroom,
                device_type: DeviceType::Recuperator,
                action_type: ActionType::SetIsEnabled(true),
            },
            id,
        );

        let json = json!({
            "elizabeth": [
                {
                    "room": "bathroom",
                    "device_type": "recuperator",
                    "action_type": {
                        "set_is_enabled": true
                    }
                },
                "cff182e2-2bcb-4c19-a070-43d43ef7c104"
            ]
        });

        let deserialized: Action = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, action);
    }
}
