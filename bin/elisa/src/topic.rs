use paho_mqtt::QOS_1;
use topics::{Device, Topic, TopicType};

use crate::{Action, State};

pub const fn topic_for_action(feature: Action) -> Topic<Action> {
    Topic {
        topic_type: TopicType::Action,
        room: None,
        device: Device::VacuumCleaner,
        feature,
    }
}

pub const fn topic_for_state(feature: State) -> Topic<State> {
    Topic {
        topic_type: TopicType::State,
        room: None,
        device: Device::VacuumCleaner,
        feature,
    }
}

pub fn actions_topics_and_qos() -> ([String; 5], [i32; 10]) {
    (
        [
            topic_for_action(Action::Start).to_string(),
            topic_for_action(Action::Stop).to_string(),
            topic_for_action(Action::SetFanSpeed).to_string(),
            topic_for_action(Action::Pause).to_string(),
            topic_for_action(Action::Resume).to_string(),
        ],
        [QOS_1; 10],
    )
}
