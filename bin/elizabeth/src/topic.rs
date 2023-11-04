use crate::{
    Device::{self, *},
    State::{self, *},
    StatePayload,
};
use inspinia::Room::*;
use paho_mqtt::QOS_1;

use topics::{Topic, TopicType};

pub fn set_topics_and_qos() -> ([String; 10], [i32; 10]) {
    (
        [
            topic_for_state(IsEnabled, LivingRoom, Recuperator).to_string(),
            topic_for_state(FanSpeed, LivingRoom, Recuperator).to_string(),
            topic_for_state(IsEnabled, Bedroom, Thermostat).to_string(),
            topic_for_state(Temperature, Bedroom, Thermostat).to_string(),
            topic_for_state(IsEnabled, HomeOffice, Thermostat).to_string(),
            topic_for_state(Temperature, HomeOffice, Thermostat).to_string(),
            topic_for_state(IsEnabled, LivingRoom, Thermostat).to_string(),
            topic_for_state(Temperature, LivingRoom, Thermostat).to_string(),
            topic_for_state(IsEnabled, Nursery, Thermostat).to_string(),
            topic_for_state(Temperature, Nursery, Thermostat).to_string(),
        ],
        [QOS_1; 10],
    )
}

pub const fn topic_for_state(state: State, room: inspinia::Room, device: Device) -> Topic<State> {
    Topic {
        topic_type: TopicType::State,
        room: Some(map_room(room)),
        device,
        feature: state,
    }
}

impl From<StatePayload> for Topic<State> {
    fn from(val: StatePayload) -> Self {
        Topic {
            topic_type: TopicType::State,
            room: Some(map_room(val.room)),
            device: val.device,
            feature: val.state,
        }
    }
}

const fn map_room(room: inspinia::Room) -> topics::Room {
    match room {
        inspinia::Room::LivingRoom => topics::Room::LivingRoom,
        inspinia::Room::Bedroom => topics::Room::Bedroom,
        inspinia::Room::HomeOffice => topics::Room::HomeOffice,
        inspinia::Room::Nursery => topics::Room::Nursery,
    }
}
