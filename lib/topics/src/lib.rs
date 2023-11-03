mod elisa;
mod elizabeth;

pub use elisa::Action as ElisaAction;
pub use elisa::State as ElisaState;
pub use elizabeth::State as ElizabethState;

use std::fmt::{self, Display};
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

trait Feature: FromStr + Display {
    fn service() -> Service;
}

#[derive(Debug)]
pub struct Topic<F> {
    pub topic_type: TopicType,
    pub room: Option<Room>,
    pub device: Device,
    pub feature: F,
}

impl<F: Feature> fmt::Display for Topic<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let room = if let Some(room) = &self.room {
            room.to_string()
        } else {
            "none".to_string()
        };

        write!(
            f,
            "{}/{}/{}/{}/{}",
            F::service(),
            self.topic_type,
            room,
            self.device,
            self.feature
        )
    }
}

impl<F: FromStr<Err = value::Error> + Feature> FromStr for Topic<F> {
    type Err = value::Error;

    fn from_str(s: &str) -> std::result::Result<Topic<F>, Self::Err>
    where
        F::Err: serde::de::Error,
    {
        let mut split = s.split('/');

        let service = split
            .next()
            .ok_or(value::Error::custom("missing service"))?;
        let service = Service::from_str(service)?;

        if F::service() != service {
            return Err(value::Error::custom(format!(
                "expected service {}, got {}",
                F::service(),
                service
            )));
        }

        let topic_type = split
            .next()
            .ok_or(value::Error::custom("missing topic type"))?;
        let topic_type = TopicType::from_str(topic_type)?;

        let room = split.next().ok_or(value::Error::custom("missing room"))?;
        let room = if room == "none" {
            None
        } else {
            Some(Room::from_str(room)?)
        };

        let device = split
            .next()
            .ok_or(value::Error::custom("missing device type"))?;
        let device = Device::from_str(device)?;

        let feature = split
            .next()
            .ok_or(value::Error::custom("missing feature"))?;
        let feature = F::from_str(feature)?;

        Ok(Self {
            topic_type,
            room,
            device,
            feature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Str, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum ElisaFeature {
        Start,
        Stop,
    }

    impl Feature for ElisaFeature {
        fn service() -> Service {
            Service::Elisa
        }
    }

    #[derive(Debug, Deserialize, Serialize, Str, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum ElizabethFeature {
        IsEnabled,
        Temperature,
    }

    impl Feature for ElizabethFeature {
        fn service() -> Service {
            Service::Elizabeth
        }
    }

    #[test]
    fn test_from_str() {
        let topic: Topic<ElisaFeature> = "elisa/action/bathroom/thermostat/start".parse().unwrap();

        assert_eq!(topic.topic_type, TopicType::Action);
        assert_eq!(topic.room, Some(Room::Bathroom));
        assert_eq!(topic.device, Device::Thermostat);
        assert_eq!(topic.feature, ElisaFeature::Start);

        let topic: Topic<ElizabethFeature> = "elizabeth/state/hallway/thermostat/is_enabled"
            .parse()
            .unwrap();

        assert_eq!(topic.topic_type, TopicType::State);
        assert_eq!(topic.room, Some(Room::Hallway));
        assert_eq!(topic.device, Device::Thermostat);
        assert_eq!(topic.feature, ElizabethFeature::IsEnabled);
    }

    #[test]
    fn test_to_string() {
        let topic = Topic::<ElisaFeature> {
            topic_type: TopicType::Action,
            room: Some(Room::Bathroom),
            device: Device::Thermostat,
            feature: ElisaFeature::Start,
        };

        assert_eq!(topic.to_string(), "elisa/action/bathroom/thermostat/start");

        let topic = Topic::<ElizabethFeature> {
            topic_type: TopicType::State,
            room: Some(Room::Hallway),
            device: Device::Thermostat,
            feature: ElizabethFeature::Temperature,
        };

        assert_eq!(
            topic.to_string(),
            "elizabeth/state/hallway/thermostat/temperature"
        );
    }
}
