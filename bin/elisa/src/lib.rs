use std::sync::Arc;

use roborock::{CleanupMode as RoborockCleanupMode, FanSpeed, Status, Vacuum};
use tokio::sync::{mpsc, oneshot};
use transport::{
    action::{ActionRequest, ActionResponse, ActionResult},
    elisa::{Action, CleanupMode, State, WorkSpeed},
    state::{StateRequest, StateResponse},
    DeviceType,
};

use log::{debug, error, info};
use paho_mqtt::{AsyncClient as MqClient, Message, MessageBuilder, PropertyCode};

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

enum VacuumRequest {
    Action(Action, oneshot::Sender<Result<()>>),
    Status(oneshot::Sender<Result<(Status, Vec<u8>)>>),
}

#[derive(Clone)]
pub struct VacuumQueue {
    tx: mpsc::Sender<VacuumRequest>,
}

impl VacuumQueue {
    pub fn new(mut vacuum: Vacuum) -> Self {
        let (tx, mut rx) = mpsc::channel(16);

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                match request {
                    VacuumRequest::Action(action, responder) => {
                        let _ = responder.send(perform_action(action, &mut vacuum).await);
                    }
                    VacuumRequest::Status(responder) => {
                        let result = vacuum
                            .status()
                            .await
                            .map(|status| (status, vacuum.last_cleaning_rooms().to_vec()))
                            .map_err(Error::from);
                        let _ = responder.send(result);
                    }
                }
            }
        });

        Self { tx }
    }

    pub async fn run_action(&self, action: Action) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(VacuumRequest::Action(action, tx))
            .await
            .map_err(|_| Error::QueueClosed)?;
        rx.await.map_err(|_| Error::QueueClosed)?
    }

    pub async fn get_status(&self) -> Result<(Status, Vec<u8>)> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(VacuumRequest::Status(tx))
            .await
            .map_err(|_| Error::QueueClosed)?;
        rx.await.map_err(|_| Error::QueueClosed)?
    }
}

pub async fn handle_action_request(msg: Message, mqtt: &mut MqClient, vacuum: Arc<VacuumQueue>) {
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
            let result = match vacuum.run_action(action).await {
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

pub async fn handle_state_request(msg: Message, mqtt: &mut MqClient, vacuum: Arc<VacuumQueue>) {
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

    let should_respond = request
        .device_ids
        .iter()
        .any(|id| id.device_type == DeviceType::VacuumCleaner);

    if should_respond {
        match vacuum.get_status().await {
            Ok((status, rooms)) => {
                let state = prepare_state(status, &rooms);
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
            Err(err) => {
                error!("Error fetching vacuum status: {}", err);
            }
        }
    }
}

async fn perform_action(action: Action, vacuum: &mut Vacuum) -> Result<()> {
    match action {
        Action::Start(rooms) => {
            let room_ids = rooms.iter().filter_map(room_id_for_room).collect();

            info!("wants to start cleaning in rooms: {:?}", rooms);
            vacuum.start(room_ids).await?;
            Ok(())
        }
        Action::Stop => {
            info!("wants to stop cleaning");
            vacuum.stop().await?;
            vacuum.go_home().await?;
            Ok(())
        }
        Action::SetWorkSpeed(work_speed) => {
            let mode = from_elisa_speed(work_speed);

            info!("wants to set mode {:?}", mode);
            vacuum.set_fan_speed(mode).await?;
            Ok(())
        }
        Action::SetCleanupMode(cleanup_mode) => {
            let mode = from_elisa_cleanup(cleanup_mode);

            info!("wants to set cleanup mode {:?}", mode);
            vacuum.set_cleanup_mode(mode).await?;
            Ok(())
        }
        Action::Pause => {
            info!("wants to pause");
            vacuum.pause().await?;
            Ok(())
        }
        Action::Resume => {
            info!("wants to resume");
            vacuum.resume().await?;
            Ok(())
        }
    }
}

pub fn prepare_state(status: Status, rooms: &[u8]) -> State {
    State {
        battery_level: status.battery,
        is_enabled: status.state.is_enabled(),
        is_paused: status.state.is_paused(),
        work_speed: from_roborock_speed(status.fan_speed),
        cleanup_mode: from_roborock_cleanup(status.cleanup_mode),
        rooms: rooms.iter().filter_map(room_from_id).collect(),
    }
}

fn room_id_for_room(room: &transport::Room) -> Option<u8> {
    match room {
        transport::Room::Bathroom => Some(16),
        transport::Room::Bedroom => Some(17),
        transport::Room::Corridor => Some(23),
        transport::Room::Hallway => Some(24),
        transport::Room::HomeOffice => Some(21),
        transport::Room::Kitchen => Some(19),
        transport::Room::LivingRoom => Some(18),
        transport::Room::Toilet => Some(22),
        _ => None,
    }
}

fn room_from_id(id: &u8) -> Option<transport::Room> {
    match id {
        16 => Some(transport::Room::Bathroom),
        17 => Some(transport::Room::Bedroom),
        23 => Some(transport::Room::Corridor),
        24 => Some(transport::Room::Hallway),
        21 => Some(transport::Room::HomeOffice),
        19 => Some(transport::Room::Kitchen),
        18 => Some(transport::Room::LivingRoom),
        22 => Some(transport::Room::Toilet),
        _ => None,
    }
}

fn from_roborock_speed(speed: FanSpeed) -> WorkSpeed {
    match speed {
        FanSpeed::Off => WorkSpeed::Min,
        FanSpeed::Silent => WorkSpeed::Silent,
        FanSpeed::Standard => WorkSpeed::Standard,
        FanSpeed::Medium => WorkSpeed::Medium,
        FanSpeed::Turbo => WorkSpeed::Turbo,
        FanSpeed::Max => WorkSpeed::Turbo,
        FanSpeed::SmartMode => WorkSpeed::Standard,
    }
}

fn from_elisa_speed(speed: WorkSpeed) -> FanSpeed {
    match speed {
        WorkSpeed::Min => FanSpeed::Off,
        WorkSpeed::Silent => FanSpeed::Silent,
        WorkSpeed::Standard => FanSpeed::Standard,
        WorkSpeed::Medium => FanSpeed::Medium,
        WorkSpeed::Turbo => FanSpeed::Turbo,
    }
}

fn from_roborock_cleanup(mode: RoborockCleanupMode) -> CleanupMode {
    match mode {
        RoborockCleanupMode::DryCleaning => CleanupMode::DryCleaning,
        RoborockCleanupMode::WetCleaning => CleanupMode::WetCleaning,
        RoborockCleanupMode::MixedCleaning => CleanupMode::MixedCleaning,
    }
}

fn from_elisa_cleanup(mode: CleanupMode) -> RoborockCleanupMode {
    match mode {
        CleanupMode::DryCleaning => RoborockCleanupMode::DryCleaning,
        CleanupMode::WetCleaning => RoborockCleanupMode::WetCleaning,
        CleanupMode::MixedCleaning => RoborockCleanupMode::MixedCleaning,
    }
}
