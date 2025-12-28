use std::collections::HashMap;

use sonoff::{Client, SonoffDevice};
use transport::{
    action::{ActionRequest, ActionResponse, ActionResult},
    elisheba::{Action, State},
    state::{StateRequest, StateResponse},
    DeviceType, Room,
};

use log::{debug, error, info};
use paho_mqtt::{AsyncClient as MqClient, Message, MessageBuilder, PropertyCode};

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Storage {
    lights: HashMap<Room, State>,
}

impl Storage {
    pub fn apply(&mut self, device: &SonoffDevice) -> Option<State> {
        let state = map_device_to_state(device)?;

        info!(
            "ligths at {room} are toggled {state}",
            room = state.room,
            state = if state.is_enabled { "on" } else { "off" }
        );

        self.lights.insert(state.room, state);

        Some(state)
    }
}

fn map_device_to_state(device: &SonoffDevice) -> Option<State> {
    let room = map_id_to_room(&device.id)?;
    let switch = device.meta["switches"][0]["switch"].as_str()?;
    let is_enabled = match switch {
        "on" => true,
        "off" => false,
        other => {
            error!("unknown switch state {other}");
            false
        }
    };

    Some(State { is_enabled, room })
}

pub async fn handle_action_request(msg: Message, mqtt: &mut MqClient, sonoff: &mut Client) {
    let request: ActionRequest = match serde_json::from_slice(msg.payload()) {
        Ok(ids) => ids,
        Err(err) => {
            error!("unable to parse request: {}", err);
            error!("{}", msg.payload_str());
            return;
        }
    };

    let response_topic = match msg.properties().get_string(PropertyCode::ResponseTopic) {
        Some(topic) => topic,
        None => {
            error!("missing response topic");
            return;
        }
    };

    for action in request.actions {
        if let transport::action::Action::Elisheba(action, action_id) = action {
            let result = match update_state(action, sonoff).await {
                Ok(_) => ActionResult::Success,
                Err(err) => {
                    error!("Error updating state: {}", err);
                    ActionResult::Failure
                }
            };

            let response = ActionResponse { action_id, result };

            debug!("publish to {}: {:?}", response_topic, response);

            let payload = serde_json::to_vec(&response).unwrap();

            let message = MessageBuilder::new()
                .topic(&response_topic)
                .payload(payload)
                .finalize();

            match mqtt.publish(message).await {
                Ok(()) => (),
                Err(err) => {
                    error!("Error sending response to {}: {}", response_topic, err);
                }
            }
        }
    }
}

async fn update_state(action: Action, sonoff: &mut Client) -> Result<()> {
    let state = if action.is_enabled { "on" } else { "off" };

    info!("wants to toggle {state} lights at {}", action.room);

    let device_id = map_room_to_id(action.room).ok_or(Error::UnknownDevice)?;
    sonoff.update_state(device_id, action.is_enabled).await?;

    info!("successfully toggled {state} at {}", action.room);

    Ok(())
}

pub async fn handle_state_request(msg: Message, mqtt: &mut MqClient, sonoff: &mut Client) {
    let request: StateRequest = match serde_json::from_slice(msg.payload()) {
        Ok(ids) => ids,
        Err(err) => {
            error!("unable to parse request: {}", err);
            error!("{}", msg.payload_str());
            return;
        }
    };

    let response_topic = match msg.properties().get_string(PropertyCode::ResponseTopic) {
        Some(topic) => topic,
        None => {
            error!("missing response topic");
            return;
        }
    };

    let ids = request
        .device_ids
        .into_iter()
        .filter(|id| matches!(id.device_type, DeviceType::Light));

    debug!("ids: {:?}", ids.clone().collect::<Vec<_>>());

    for id in ids {
        let state = match get_state(id.room, sonoff).await {
            Ok(state) => state,
            Err(err) => {
                error!("Error getting state for {id}: {}", err);
                continue;
            }
        };

        debug!("publish to {}: {:?}", response_topic, state);

        let response = StateResponse::Elisheba(state);
        let payload = serde_json::to_vec(&response).unwrap();

        let message = MessageBuilder::new()
            .topic(&response_topic)
            .payload(payload)
            .finalize();

        match mqtt.publish(message).await {
            Ok(()) => (),
            Err(err) => {
                error!("Error sending response to {}: {}", response_topic, err);
            }
        }
    }
}

async fn get_state(room: Room, sonoff: &mut Client) -> Result<State> {
    let device_id = map_room_to_id(room).ok_or(Error::UnknownDevice)?;
    let device = sonoff.get_state(device_id).await?;
    let state = map_device_to_state(&device).ok_or(Error::UnknownDevice)?;
    Ok(state)
}

fn map_id_to_room(id: &str) -> Option<Room> {
    match id {
        "1002074ed2" => Some(Room::Corridor),
        "10020750eb" => Some(Room::Nursery),
        _ => None,
    }
}

fn map_room_to_id(room: Room) -> Option<&'static str> {
    match room {
        Room::Corridor => Some("1002074ed2"),
        Room::Nursery => Some("10020750eb"),
        _ => None,
    }
}
