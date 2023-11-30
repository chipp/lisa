use std::fmt;
use std::str::FromStr;

use serde::de::{value, Error};

#[derive(Debug, PartialEq)]
pub enum Topic {
    State,
    StateRequest,
    StateResponse(String),
    ActionRequest,
    ActionResponse(String),
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Topic::State => write!(f, "state"),
            Topic::StateRequest => write!(f, "state/request"),
            Topic::StateResponse(device_id) => write!(f, "state/response/{}", device_id),
            Topic::ActionRequest => write!(f, "action/request"),
            Topic::ActionResponse(device_id) => write!(f, "action/response/{}", device_id),
        }
    }
}

impl FromStr for Topic {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Topic, Self::Err> {
        const ERROR_MSG: &str = "supported topics are state, state/request, action/request, \
            state/response/<id> and action/response/<id>";

        match s {
            "state" => Ok(Topic::State),
            "state/request" => Ok(Topic::StateRequest),
            "action/request" => Ok(Topic::ActionRequest),
            _ => {
                let (topic, id) = s
                    .rsplit_once('/')
                    .ok_or_else(|| value::Error::custom(ERROR_MSG))?;

                match topic {
                    "state/response" => Ok(Topic::StateResponse(id.to_string())),
                    "action/response" => Ok(Topic::ActionResponse(id.to_string())),
                    _ => Err(value::Error::custom(ERROR_MSG)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let topic = Topic::State;
        assert_eq!(topic.to_string(), "state");

        let topic = Topic::StateRequest;
        assert_eq!(topic.to_string(), "state/request");

        let topic = Topic::StateResponse("656997EA-01B8-4F64-84E8-9603F12FD448".to_string());
        assert_eq!(
            topic.to_string(),
            "state/response/656997EA-01B8-4F64-84E8-9603F12FD448"
        );

        let topic = Topic::ActionRequest;
        assert_eq!(topic.to_string(), "action/request");

        let topic = Topic::ActionResponse("AD56F627-ABF2-4F3C-B098-FF8D76DE4F72".to_string());
        assert_eq!(
            topic.to_string(),
            "action/response/AD56F627-ABF2-4F3C-B098-FF8D76DE4F72"
        );
    }

    #[test]
    fn test_deserialization() {
        let topic = Topic::from_str("state").unwrap();
        assert_eq!(topic, Topic::State);

        let topic = Topic::from_str("state/request").unwrap();
        assert_eq!(topic, Topic::StateRequest);

        let topic = Topic::from_str("state/response/8E1E559B-27A4-4D2D-990F-AE8D5FF9B074").unwrap();
        assert_eq!(
            topic,
            Topic::StateResponse("8E1E559B-27A4-4D2D-990F-AE8D5FF9B074".to_string())
        );

        let topic = Topic::from_str("action/request").unwrap();
        assert_eq!(topic, Topic::ActionRequest);

        let topic =
            Topic::from_str("action/response/D4A49F18-5F23-4848-B3CB-1A37E206D64E").unwrap();
        assert_eq!(
            topic,
            Topic::ActionResponse("D4A49F18-5F23-4848-B3CB-1A37E206D64E".to_string())
        );
    }
}
