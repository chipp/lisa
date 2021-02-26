use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    Start { rooms: Vec<u8> },
    SetMode { mode: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::*;

    #[test]
    fn test_start() {
        let command = Command::Start {
            rooms: vec![11, 12, 13],
        };

        assert_eq!(
            to_value(&command).unwrap(),
            json!({
                "type": "start",
                "rooms": [11, 12, 13]
            })
        );

        match from_value(json!({
            "type": "start",
            "rooms": [11, 12, 13]
        }))
        .unwrap()
        {
            Command::Start { rooms } => assert_eq!(rooms, &[11, 12, 13]),
            Command::SetMode { mode: _ } => panic!(),
        }
    }

    #[test]
    fn test_set_mode() {
        let command = Command::SetMode {
            mode: "quiet".to_string(),
        };

        assert_eq!(
            to_value(&command).unwrap(),
            json!({
                "type": "set_mode",
                "mode": "quiet"
            })
        );

        match from_value(json!({
            "type": "set_mode",
            "mode": "quiet"
        }))
        .unwrap()
        {
            Command::Start { rooms: _ } => panic!(),
            Command::SetMode { mode } => assert_eq!(mode, "quiet"),
        }
    }
}
