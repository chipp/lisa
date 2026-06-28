use std::sync::Arc;

use log::{debug, error, info};
use paho_mqtt::{AsyncClient as MqClient, Message, MessageBuilder, PropertyCode};
use tokio::sync::Mutex;
use transport::{
    action::{ActionRequest, ActionResponse, ActionResult},
    elzhbieta::State,
    state::{StateRequest, StateResponse},
    DeviceType,
};

mod error;
pub use error::{Error, Result};

pub type SharedClient = Arc<Mutex<tuya::Client>>;

pub async fn handle_action_request(msg: Message, mqtt: &mut MqClient, client: SharedClient) {
    let request: ActionRequest = match serde_json::from_slice(msg.payload()) {
        Ok(request) => request,
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
        if let transport::action::Action::Elzhbieta(action, action_id) = action {
            let result = {
                let mut client = client.lock().await;
                client.apply(action).await
            };

            let result = match result {
                Ok(()) => ActionResult::Success,
                Err(err) => {
                    error!("Error updating AC state: {}", err);
                    ActionResult::Failure
                }
            };

            publish_action_response(mqtt, &response_topic, ActionResponse { action_id, result })
                .await;
        }
    }
}

pub async fn handle_state_request(msg: Message, mqtt: &mut MqClient, client: SharedClient) {
    let request: StateRequest = match serde_json::from_slice(msg.payload()) {
        Ok(request) => request,
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

    let rooms = request
        .device_ids
        .into_iter()
        .filter(|id| id.device_type == DeviceType::AirConditioner)
        .map(|id| id.room)
        .collect::<Vec<_>>();

    for room in rooms {
        let state = {
            let mut client = client.lock().await;
            client.state(room).await
        };

        match state {
            Ok(state) => publish_state_response(mqtt, &response_topic, state).await,
            Err(err) => error!("Error getting AC state for {room}: {}", err),
        }
    }
}

pub async fn publish_state_update(mqtt: &MqClient, state: State) {
    let update = transport::state::StateUpdate::Elzhbieta(state);
    let payload = match serde_json::to_vec(&update) {
        Ok(payload) => payload,
        Err(err) => {
            error!("Error serializing state update: {err}");
            return;
        }
    };

    let message = MessageBuilder::new()
        .topic(transport::Topic::StateUpdate.to_string())
        .payload(payload)
        .finalize();

    if let Err(err) = mqtt.publish(message).await {
        error!("Error publishing state update: {err}");
    }
}

pub async fn publish_all_states(mqtt: &MqClient, client: SharedClient) {
    let rooms = {
        let client = client.lock().await;
        client.rooms().collect::<Vec<_>>()
    };

    for room in rooms {
        let state = {
            let mut client = client.lock().await;
            client.state(room).await
        };

        match state {
            Ok(state) => {
                info!("publishing AC state: {:?}", state);
                publish_state_update(mqtt, state).await;
            }
            Err(err) => error!("Error polling AC state for {room}: {}", err),
        }
    }
}

async fn publish_action_response(mqtt: &mut MqClient, topic: &str, response: ActionResponse) {
    debug!("publish to {}: {:?}", topic, response);
    let payload = serde_json::to_vec(&response).unwrap();
    let message = MessageBuilder::new()
        .topic(topic)
        .payload(payload)
        .finalize();

    if let Err(err) = mqtt.publish(message).await {
        error!("Error sending action response to {}: {}", topic, err);
    }
}

async fn publish_state_response(mqtt: &mut MqClient, topic: &str, state: State) {
    debug!("publish to {}: {:?}", topic, state);
    let response = StateResponse::Elzhbieta(state);
    let payload = serde_json::to_vec(&response).unwrap();
    let message = MessageBuilder::new()
        .topic(topic)
        .payload(payload)
        .finalize();

    if let Err(err) = mqtt.publish(message).await {
        error!("Error sending state response to {}: {}", topic, err);
    }
}
