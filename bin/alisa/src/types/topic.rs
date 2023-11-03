use topics::{
    Device::{self, *},
    ElisaState, ElizabethState,
    Room::{self, *},
    Topic, TopicType,
};

use paho_mqtt::QOS_1;

use crate::Action;

pub fn create_action_topic(device: Device, room: Room, action: Action) -> String {
    match action {
        Action::Elizabeth(action) => Topic {
            topic_type: TopicType::Action,
            room: Some(room),
            device,
            feature: action,
        }
        .to_string(),
        Action::Elisa(action) => Topic {
            topic_type: TopicType::Action,
            room: Some(room),
            device,
            feature: action,
        }
        .to_string(),
    }
}

const fn elizabeth_topic(
    room: Room,
    device: Device,
    state: ElizabethState,
) -> Topic<ElizabethState> {
    Topic {
        topic_type: TopicType::State,
        room: Some(room),
        device,
        feature: state,
    }
}

const fn elisa_topic() -> Topic<ElisaState> {
    Topic {
        topic_type: TopicType::State,
        room: None,
        device: Device::VacuumCleaner,
        feature: ElisaState::Status,
    }
}

pub fn state_topics_and_qos() -> ([String; 11], [i32; 11]) {
    (
        [
            elizabeth_topic(LivingRoom, Recuperator, ElizabethState::IsEnabled).to_string(),
            elizabeth_topic(LivingRoom, Recuperator, ElizabethState::FanSpeed).to_string(),
            elizabeth_topic(Bedroom, Thermostat, ElizabethState::IsEnabled).to_string(),
            elizabeth_topic(Bedroom, Thermostat, ElizabethState::Temperature).to_string(),
            elizabeth_topic(HomeOffice, Thermostat, ElizabethState::IsEnabled).to_string(),
            elizabeth_topic(HomeOffice, Thermostat, ElizabethState::Temperature).to_string(),
            elizabeth_topic(LivingRoom, Thermostat, ElizabethState::IsEnabled).to_string(),
            elizabeth_topic(LivingRoom, Thermostat, ElizabethState::Temperature).to_string(),
            elizabeth_topic(Nursery, Thermostat, ElizabethState::IsEnabled).to_string(),
            elizabeth_topic(Nursery, Thermostat, ElizabethState::Temperature).to_string(),
            elisa_topic().to_string(),
        ],
        [QOS_1; 11],
    )
}
