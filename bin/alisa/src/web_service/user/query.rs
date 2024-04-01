use std::collections::HashSet;
use std::time::Duration;

use alice::{StateDevice, StateRequest, StateResponse};
use transport::{connect_mqtt, DeviceId, DeviceType, Topic};

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Result};
use axum::Json;
use futures_util::StreamExt;
use log::{debug, info};
use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode, QOS_1};

use crate::reporter;
use crate::web_service::auth::validate_autorization;
use crate::web_service::ServiceError;

pub async fn query(
    headers: HeaderMap,
    Json(query): Json<StateRequest>,
) -> Result<impl IntoResponse> {
    validate_autorization(&headers, "devices_query")?;

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");

    let mut mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "alisa_query")
        .await
        .expect("failed to connect mqtt");

    let request_id = headers.get("X-Request-Id").unwrap().to_str().unwrap();

    let device_ids = query
        .devices
        .iter()
        .map(|device| device.id)
        .collect::<Vec<_>>();

    info!(
        "{request_id}/query ({})",
        device_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    let request = transport::state::StateRequest { device_ids };

    let request_topic = Topic::StateRequest.to_string();
    let response_topic = Topic::StateResponse(request_id.to_string()).to_string();

    mqtt_client.subscribe(&response_topic, QOS_1);
    let mut stream = mqtt_client.get_stream(1);

    let mut props = Properties::new();
    props
        .push_string(PropertyCode::ResponseTopic, &response_topic)
        .map_err(ServiceError::from)?;

    debug!("request to {}: {:?}", request_topic, request);
    debug!("waiting for response on {}", response_topic);

    let payload = serde_json::to_vec(&request).map_err(ServiceError::from)?;
    let request_msg = MessageBuilder::new()
        .topic(request_topic)
        .properties(props)
        .payload(payload)
        .finalize();

    mqtt_client.publish(request_msg);

    let mut device_ids: HashSet<_> = HashSet::from_iter(request.device_ids);
    let mut devices = vec![];

    while let Ok(Some(msg_opt)) = tokio::time::timeout(Duration::from_secs(3), stream.next()).await
    {
        if let Some(msg) = msg_opt {
            debug!("msg: {:?}", msg);
            debug!("msg_str: {:?}", msg.payload_str());

            handle_message(msg, &mut device_ids, &mut devices)?;
        }

        if device_ids.is_empty() {
            break;
        }
    }

    mqtt_client.stop_stream();
    mqtt_client.unsubscribe(&response_topic);

    for device_id in device_ids {
        devices.push(StateDevice::new_empty(device_id));
    }

    let response = StateResponse::new(request_id.to_string(), devices);

    Ok((StatusCode::OK, Json(response)))
}

fn handle_message(
    msg: Message,
    device_ids: &mut HashSet<DeviceId>,
    devices: &mut Vec<StateDevice>,
) -> Result<()> {
    use transport::state::StateResponse;

    let response: StateResponse =
        serde_json::from_slice(msg.payload()).map_err(ServiceError::from)?;
    debug!("got response {:?}", response);

    match response {
        StateResponse::Elisa(state) => {
            let states = reporter::prepare_vacuum_updates(state);

            for state in states {
                if device_ids.contains(&state.id()) {
                    device_ids.remove(&state.id());
                    devices.push(state);
                }
            }
        }
        StateResponse::Elizabeth(state) => match state.device_type {
            DeviceType::Recuperator => {
                let state = reporter::prepare_recuperator_current_state(state);

                if device_ids.contains(&state.id()) {
                    device_ids.remove(&state.id());
                    devices.push(state);
                }
            }
            DeviceType::Thermostat => {
                let state = reporter::prepare_thermostat_current_state(state);

                if device_ids.contains(&state.id()) {
                    device_ids.remove(&state.id());
                    devices.push(state);
                }
            }
            DeviceType::TemperatureSensor | DeviceType::VacuumCleaner | DeviceType::Light => (),
        },
        StateResponse::Elisheba(_) => todo!(),
    }

    Ok(())
}
