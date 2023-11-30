mod storage;
pub use storage::Storage;

use std::sync::Arc;

use tokio::sync::Mutex;
use transport::{
    action::{ActionRequest, ActionResponse, ActionResult},
    elisa::{Action, State, WorkSpeed},
    state::StateResponse,
    DeviceId, DeviceType,
};
use xiaomi::{FanSpeed, Status, Vacuum};

use log::{debug, error, info};
use paho_mqtt::{AsyncClient as MqClient, Message, MessageBuilder, PropertyCode};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

pub async fn handle_action_request(msg: Message, mqtt: &mut MqClient, vacuum: Arc<Mutex<Vacuum>>) {
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
        if let transport::action::Action::Elisa(action, action_id) = action {
            let vacuum = &mut vacuum.lock().await;
            let result = match perform_action(action, vacuum).await {
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

pub async fn handle_state_request(msg: Message, mqtt: &mut MqClient, vacuum: Arc<Mutex<Vacuum>>) {
    let ids: Vec<DeviceId> = match serde_json::from_slice(msg.payload()) {
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

    let should_respond = ids
        .iter()
        .any(|id| id.device_type == DeviceType::VacuumCleaner);

    if should_respond {
        let mut vacuum = vacuum.lock().await;

        if let Ok(status) = vacuum.status().await {
            let state = prepare_state(status, vacuum.last_cleaning_rooms());
            debug!("publish to {}: {:?}", response_topic, state);

            let response = StateResponse::Elisa(state);

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

async fn perform_action(action: Action, vacuum: &mut Vacuum) -> Result<()> {
    match action {
        Action::Start(rooms) => {
            let room_ids = rooms.iter().map(room_id_for_room).collect();

            info!("wants to start cleaning in rooms: {:?}", rooms);
            vacuum.start(room_ids).await
        }
        Action::Stop => {
            info!("wants to stop cleaning");
            vacuum.stop().await?;
            vacuum.go_home().await
        }
        Action::SetWorkSpeed(work_speed) => {
            let mode = from_elisa_speed(work_speed);

            info!("wants to set mode {}", mode);
            vacuum.set_fan_speed(mode).await
        }
        Action::Pause => {
            info!("wants to pause");
            vacuum.pause().await
        }
        Action::Resume => {
            info!("wants to resume");
            vacuum.resume().await
        }
    }
}

pub fn prepare_state(status: Status, rooms: &[u8]) -> State {
    State {
        battery_level: status.battery,
        is_enabled: status.state.is_enabled(),
        is_paused: status.state.is_paused(),
        work_speed: from_xiaomi_speed(status.fan_speed),
        rooms: rooms.iter().filter_map(room_from_id).collect(),
    }
}

fn room_id_for_room(room: &transport::Room) -> u8 {
    match room {
        // TODO: read configuration to config file
        transport::Room::Bathroom => 11,
        transport::Room::Bedroom => 13,
        transport::Room::Corridor => 15,
        transport::Room::Hallway => 12,
        transport::Room::HomeOffice => 17,
        transport::Room::Kitchen => 16,
        transport::Room::LivingRoom => 18,
        transport::Room::Nursery => 14,
        transport::Room::Toilet => 10,
    }
}

fn room_from_id(id: &u8) -> Option<transport::Room> {
    match id {
        11 => Some(transport::Room::Bathroom),
        13 => Some(transport::Room::Bedroom),
        15 => Some(transport::Room::Corridor),
        12 => Some(transport::Room::Hallway),
        17 => Some(transport::Room::HomeOffice),
        16 => Some(transport::Room::Kitchen),
        18 => Some(transport::Room::LivingRoom),
        14 => Some(transport::Room::Nursery),
        10 => Some(transport::Room::Toilet),
        _ => None,
    }
}

fn from_xiaomi_speed(speed: FanSpeed) -> WorkSpeed {
    match speed {
        FanSpeed::Silent => WorkSpeed::Silent,
        FanSpeed::Standard => WorkSpeed::Standard,
        FanSpeed::Medium => WorkSpeed::Medium,
        FanSpeed::Turbo => WorkSpeed::Turbo,
    }
}

fn from_elisa_speed(speed: WorkSpeed) -> FanSpeed {
    match speed {
        WorkSpeed::Silent => FanSpeed::Silent,
        WorkSpeed::Standard => FanSpeed::Standard,
        WorkSpeed::Medium => FanSpeed::Medium,
        WorkSpeed::Turbo => FanSpeed::Turbo,
    }
}
