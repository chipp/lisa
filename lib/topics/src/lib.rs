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

#[derive(Debug, Deserialize, Serialize, Str, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    Elisa,
    Elizabeth,
}

#[derive(Debug, Deserialize, Serialize, Str, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TopicType {
    Action,
    State,
}

#[derive(Debug, Deserialize, Serialize, Str, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Device {
    Recuperator,
    TemperatureSensor,
    Thermostat,
    VacuumCleaner,
}

#[derive(Debug, Deserialize, Serialize, Str, PartialEq)]
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

trait Feature: FromStr + Display {}

#[derive(Debug)]
pub struct Topic<F> {
    pub service: Service,
    pub topic_type: TopicType,
    pub room: Option<Room>,
    pub device: Device,
    pub feature: F,
}

impl<F: Display> fmt::Display for Topic<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let room = if let Some(room) = &self.room {
            room.to_string()
        } else {
            "none".to_string()
        };

        write!(
            f,
            "{}/{}/{}/{}/{}",
            self.service, self.topic_type, room, self.device, self.feature
        )
    }
}

impl<F: FromStr<Err = value::Error>> FromStr for Topic<F> {
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
            service,
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
    enum TestFeature {
        Start,
        Stop,
    }

    #[test]
    fn test_from_str() {
        let topic: Topic<TestFeature> = "elisa/action/bathroom/thermostat/start".parse().unwrap();

        assert_eq!(topic.service, Service::Elisa);
        assert_eq!(topic.topic_type, TopicType::Action);
        assert_eq!(topic.room, Some(Room::Bathroom));
        assert_eq!(topic.device, Device::Thermostat);
        assert_eq!(topic.feature, TestFeature::Start);

        let topic: Topic<TestFeature> = "elizabeth/state/hallway/vacuum_cleaner/stop"
            .parse()
            .unwrap();

        assert_eq!(topic.service, Service::Elizabeth);
        assert_eq!(topic.topic_type, TopicType::State);
        assert_eq!(topic.room, Some(Room::Hallway));
        assert_eq!(topic.device, Device::VacuumCleaner);
        assert_eq!(topic.feature, TestFeature::Stop);
    }

    #[test]
    fn test_to_string() {
        let topic = Topic::<TestFeature> {
            service: Service::Elisa,
            topic_type: TopicType::Action,
            room: Some(Room::Bathroom),
            device: Device::Thermostat,
            feature: TestFeature::Start,
        };

        assert_eq!(topic.to_string(), "elisa/action/bathroom/thermostat/start");

        // test all variants
        let topic = Topic::<TestFeature> {
            service: Service::Elizabeth,
            topic_type: TopicType::State,
            room: Some(Room::Hallway),
            device: Device::VacuumCleaner,
            feature: TestFeature::Stop,
        };

        assert_eq!(
            topic.to_string(),
            "elizabeth/state/hallway/vacuum_cleaner/stop"
        );
    }
}
