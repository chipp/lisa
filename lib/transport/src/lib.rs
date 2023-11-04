pub mod elisa;
pub mod elizabeth;

use std::fmt;
use std::str::FromStr;

use serde::{
    de::{value, Error},
    Deserialize, Serialize,
};
use str_derive::Str;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    Elisa,
    Elizabeth,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TopicType {
    Action,
    State,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Device {
    Recuperator,
    TemperatureSensor,
    Thermostat,
    VacuumCleaner,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Bathroom,
    Bedroom,
    Corridor,
    Hallway,
    HomeOffice,
    Kitchen,
    LivingRoom,
    Nursery,
    Toilet,
}

impl Room {
    pub const fn all_rooms() -> [Room; 9] {
        [
            Room::Bathroom,
            Room::Bedroom,
            Room::Corridor,
            Room::Hallway,
            Room::HomeOffice,
            Room::Kitchen,
            Room::LivingRoom,
            Room::Nursery,
            Room::Toilet,
        ]
    }
}

#[derive(Debug, PartialEq)]
pub struct Topic {
    pub service: Service,
    pub topic_type: TopicType,
}

impl Topic {
    pub const fn elisa_action() -> Topic {
        Topic {
            service: Service::Elisa,
            topic_type: TopicType::Action,
        }
    }

    pub const fn elisa_state() -> Topic {
        Topic {
            service: Service::Elisa,
            topic_type: TopicType::State,
        }
    }

    pub const fn elizabeth_action() -> Topic {
        Topic {
            service: Service::Elizabeth,
            topic_type: TopicType::Action,
        }
    }

    pub const fn elizabeth_state() -> Topic {
        Topic {
            service: Service::Elizabeth,
            topic_type: TopicType::State,
        }
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.service, self.topic_type,)
    }
}

impl FromStr for Topic {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Topic, Self::Err> {
        let (service, topic_type) = s.split_once('/').ok_or(value::Error::custom(
            "topic should have to components e.g. service/topic_type",
        ))?;

        let service = Service::from_str(service)?;
        let topic_type = TopicType::from_str(topic_type)?;

        Ok(Self {
            service,
            topic_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            Topic::from_str("elizabeth/action").unwrap(),
            Topic {
                service: Service::Elizabeth,
                topic_type: TopicType::Action,
            }
        );

        assert_eq!(
            Topic::from_str("elizabeth/state").unwrap(),
            Topic {
                service: Service::Elizabeth,
                topic_type: TopicType::State,
            }
        );

        assert_eq!(
            Topic::from_str("elisa/action").unwrap(),
            Topic {
                service: Service::Elisa,
                topic_type: TopicType::Action,
            }
        );

        assert_eq!(
            Topic::from_str("elisa/state").unwrap(),
            Topic {
                service: Service::Elisa,
                topic_type: TopicType::State,
            }
        );

        assert!(Topic::from_str("lisa/action").is_err());
        assert!(Topic::from_str("elizabeth/st").is_err());
        assert!(Topic::from_str("elizabeth").is_err());
    }

    #[test]
    fn test_to_string() {
        assert_eq!(
            Topic {
                service: Service::Elizabeth,
                topic_type: TopicType::Action,
            }
            .to_string(),
            "elizabeth/action"
        );

        assert_eq!(
            Topic {
                service: Service::Elizabeth,
                topic_type: TopicType::State,
            }
            .to_string(),
            "elizabeth/state"
        );

        assert_eq!(
            Topic {
                service: Service::Elisa,
                topic_type: TopicType::Action,
            }
            .to_string(),
            "elisa/action"
        );

        assert_eq!(
            Topic {
                service: Service::Elisa,
                topic_type: TopicType::State,
            }
            .to_string(),
            "elisa/state"
        );
    }
}
