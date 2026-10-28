use log::{debug, error};
use paho_mqtt::{AsyncClient as MqClient, Message, MessageBuilder, PropertyCode};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use transport::{
    action::{Action, ActionRequest, ActionResponse, ActionResult},
    elizaveta::{ActionType, FanSpeed, State},
    state::{StateRequest, StateResponse, StateUpdate},
    DeviceId, Room, Topic,
};

mod error;
pub use error::{Error, Result};
pub type SharedClient = Arc<Mutex<modbus::Client>>;

async fn state(client: &modbus::Client) -> io::Result<State> {
    let is_enabled = client.read_coil(0).await?;
    let fan_speed = match client.read_holding_register(2).await? {
        1 => FanSpeed::Low,
        2 => FanSpeed::Medium,
        3 => FanSpeed::High,
        value => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected fan speed: {value}"),
            ))
        }
    };
    Ok(State {
        room: Room::LivingRoom,
        is_enabled,
        fan_speed,
    })
}

pub async fn handle_action_request(msg: Message, mqtt: &mut MqClient, client: SharedClient) {
    let request: ActionRequest = match serde_json::from_slice(msg.payload()) {
        Ok(request) => request,
        Err(err) => {
            error!("unable to parse request: {err}");
            return;
        }
    };
    let Some(topic) = msg.properties().get_string(PropertyCode::ResponseTopic) else {
        error!("missing response topic");
        return;
    };
    for action in request.actions {
        if let Action::Elizaveta(action, action_id) = action {
            let result = if action.room != Room::LivingRoom {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown recuperator room",
                ))
            } else {
                let client = client.lock().await;
                match action.action_type {
                    ActionType::SetIsEnabled(value) => client.write_coil(0, value).await,
                    ActionType::SetFanSpeed(speed) => {
                        client
                            .write_holding_register(
                                2,
                                match speed {
                                    FanSpeed::Low => 1,
                                    FanSpeed::Medium => 2,
                                    FanSpeed::High => 3,
                                },
                            )
                            .await
                    }
                }
            };
            let result = match result {
                Ok(()) => ActionResult::Success,
                Err(err) => {
                    error!("error updating recuperator: {err}");
                    ActionResult::Failure
                }
            };
            let payload = serde_json::to_vec(&ActionResponse { action_id, result }).unwrap();
            if let Err(err) = mqtt
                .publish(
                    MessageBuilder::new()
                        .topic(&topic)
                        .payload(payload)
                        .finalize(),
                )
                .await
            {
                error!("error publishing action response: {err}");
            }
        }
    }
}

pub async fn handle_state_request(msg: Message, mqtt: &mut MqClient, client: SharedClient) {
    let request: StateRequest = match serde_json::from_slice(msg.payload()) {
        Ok(request) => request,
        Err(err) => {
            error!("unable to parse request: {err}");
            return;
        }
    };
    let Some(topic) = msg.properties().get_string(PropertyCode::ResponseTopic) else {
        error!("missing response topic");
        return;
    };
    if !request
        .device_ids
        .contains(&DeviceId::recuperator_at_room(Room::LivingRoom))
    {
        return;
    }
    let result = state(&*client.lock().await).await;
    match result {
        Ok(state) => {
            let payload = serde_json::to_vec(&StateResponse::Elizaveta(state)).unwrap();
            if let Err(err) = mqtt
                .publish(
                    MessageBuilder::new()
                        .topic(topic)
                        .payload(payload)
                        .finalize(),
                )
                .await
            {
                error!("error publishing state response: {err}");
            }
        }
        Err(err) => error!("error reading recuperator: {err}"),
    }
}

pub async fn publish_all_states(mqtt: &MqClient, client: SharedClient) {
    let result = state(&*client.lock().await).await;
    match result {
        Ok(state) => {
            debug!("publishing recuperator state: {state:?}");
            let payload = serde_json::to_vec(&StateUpdate::Elizaveta(state)).unwrap();
            if let Err(err) = mqtt
                .publish(
                    MessageBuilder::new()
                        .topic(Topic::StateUpdate.to_string())
                        .payload(payload)
                        .finalize(),
                )
                .await
            {
                error!("error publishing state update: {err}");
            }
        }
        Err(err) => error!("error reading recuperator: {err}"),
    }
}
