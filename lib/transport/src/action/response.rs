use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Response {
    pub action_id: uuid::Uuid,
    pub result: ActionResult,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionResult {
    Success,
    Failure,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::uuid;

    #[test]
    fn test_serialization() {
        let id = uuid!("CFF182E2-2BCB-4C19-A070-43D43EF7C104");
        let response = Response {
            action_id: id,
            result: ActionResult::Success,
        };

        let json = json!({
            "action_id": "cff182e2-2bcb-4c19-a070-43d43ef7c104",
            "result": "success",
        });

        assert_eq!(serde_json::to_value(&response).unwrap(), json);
    }

    #[test]
    fn test_deserialization() {
        let id = uuid!("E0F2C0A6-3A3B-4C8C-9C7D-4F8E0A7D8D4F");
        let json = json!({
            "action_id": "e0f2c0a6-3a3b-4c8c-9c7d-4f8e0a7d8d4f",
            "result": "success",
        });

        let response: Response = serde_json::from_value(json).unwrap();
        assert_eq!(
            response,
            Response {
                action_id: id,
                result: ActionResult::Success,
            }
        );
    }
}
