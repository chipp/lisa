use crate::{packet::PacketContent, Packet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    Start { rooms: Vec<u8> },
    Stop,
    GoHome,
    SetWorkSpeed { mode: String },
    Pause,
    Resume,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandResponse {
    Ok,
    Failure,
}

impl PacketContent for CommandResponse {
    fn to_packet(self) -> Packet {
        Packet::CommandResponse(self)
    }
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

        let command = from_value(json!({
            "type": "start",
            "rooms": [11, 12, 13]
        }))
        .unwrap();

        match command {
            Command::Start { rooms } => assert_eq!(rooms, &[11, 12, 13]),
            _ => panic!("expected to parse Start, got {:?}", command),
        }
    }

    #[test]
    fn test_stop() {
        let command = Command::Stop;

        assert_eq!(to_value(&command).unwrap(), json!({"type": "stop"}));

        let command: Command = from_value(json!({"type": "stop"})).unwrap();

        match command {
            Command::Stop => (),
            _ => panic!("expected to parse Stop, got {:?}", command),
        }
    }

    #[test]
    fn test_go_home() {
        let command = Command::GoHome;

        assert_eq!(to_value(&command).unwrap(), json!({"type": "go_home"}));

        let command: Command = from_value(json!({"type": "go_home"})).unwrap();

        match command {
            Command::GoHome => (),
            _ => panic!("expected to parse GoHome, got {:?}", command),
        }
    }

    #[test]
    fn test_set_mode() {
        let command = Command::SetWorkSpeed {
            mode: "quiet".to_string(),
        };

        assert_eq!(
            to_value(&command).unwrap(),
            json!({
                "type": "set_work_speed",
                "mode": "quiet"
            })
        );

        let command = from_value(json!({
            "type": "set_work_speed",
            "mode": "quiet"
        }))
        .unwrap();

        match command {
            Command::SetWorkSpeed { mode } => assert_eq!(mode, "quiet"),
            _ => panic!("expected to parse SetMode, got {:?}", command),
        }
    }

    #[test]
    fn test_pause() {
        let command = Command::Pause;

        assert_eq!(to_value(&command).unwrap(), json!({"type": "pause"}));

        let command: Command = from_value(json!({"type": "pause"})).unwrap();

        match command {
            Command::Pause => (),
            _ => panic!("expected to parse Pause, got {:?}", command),
        }
    }

    #[test]
    fn test_continue() {
        let command = Command::Resume;

        assert_eq!(to_value(&command).unwrap(), json!({"type": "resume"}));

        let command: Command = from_value(json!({"type": "resume"})).unwrap();

        match command {
            Command::Resume => (),
            _ => panic!("expected to parse Continue, got {:?}", command),
        }
    }
}
