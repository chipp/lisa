mod client;
pub use client::Client;

use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use inspinia::WsError;
use log::{debug, error, info};

use paho_mqtt::{AsyncClient as MqClient, Message, MessageBuilder, PropertyCode};

use transport::action::{ActionRequest, ActionResponse, ActionResult};
use transport::elizabeth::{Action, ActionType, CurrentState};
use transport::state::{StateRequest, StateResponse};
use transport::DeviceType;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

pub async fn handle_action_request(msg: Message, mqtt: &mut MqClient, inspinia: &mut Client) {
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
        if let transport::action::Action::Elizabeth(action, action_id) = action {
            let result = match try_updating_state(action, inspinia).await {
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

fn try_updating_state(action: Action, inspinia: &mut Client) -> BoxFuture<'_, Result<()>> {
    async move {
        match update_state(action, inspinia).await {
            Ok(()) => Ok(()),
            Err(err) => match err.downcast::<WsError>() {
                Ok(err) => match *err {
                    WsError::StreamClosed | WsError::WebSocketError(_) => {
                        error!("Lost Inspinia connection. Attempting reconnect.");
                        inspinia.reconnect().await?;

                        info!("Reconnected to Inspinia!");
                        update_state(action, inspinia).await?;

                        Ok(())
                    }
                    err => Err(err.into()),
                },
                Err(err) => Err(err),
            },
        }
    }
    .boxed()
}

async fn update_state(action: Action, inspinia: &mut Client) -> Result<()> {
    debug!("Action: {:?}", action);

    match (action.device_type, action.action_type) {
        (DeviceType::Recuperator, ActionType::SetIsEnabled(value)) => {
            inspinia.set_recuperator_enabled(value).await?;
        }
        (DeviceType::Recuperator, ActionType::SetFanSpeed(speed)) => {
            inspinia.set_recuperator_fan_speed(speed).await?;
        }
        (DeviceType::Thermostat, ActionType::SetIsEnabled(value)) => {
            inspinia.set_thermostat_enabled(value, action.room).await?;
        }
        (DeviceType::Thermostat, ActionType::SetTemperature(value, relative)) => {
            if relative {
                let current = inspinia
                    .get_thermostat_temperature_in_room(action.room)
                    .await?;

                debug!("current: {}", current);
                debug!("value: {}", value);

                inspinia
                    .set_thermostat_temperature(current + value, action.room)
                    .await?;
            } else {
                inspinia
                    .set_thermostat_temperature(value, action.room)
                    .await?;
            }
        }
        _ => (),
    }

    Ok(())
}

pub async fn handle_state_request(msg: Message, mqtt: &mut MqClient, inspinia: &mut Client) {
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

    let ids = request.device_ids.into_iter().filter(|id| {
        matches!(
            id.device_type,
            DeviceType::Recuperator | DeviceType::Thermostat
        )
    });

    debug!("ids: {:?}", ids.clone().collect::<Vec<_>>());

    for id in ids {
        let capabilities = inspinia.get_current_state(id.room, id.device_type).await;

        let state = CurrentState {
            room: id.room,
            device_type: id.device_type,
            capabilities,
        };

        debug!("publish to {}: {:?}", response_topic, state);

        let response = StateResponse::Elizabeth(state);
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
