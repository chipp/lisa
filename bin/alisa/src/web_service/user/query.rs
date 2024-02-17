use std::collections::HashSet;
use std::time::Duration;

use alice::{StateDevice, StateRequest, StateResponse};
use transport::{connect_mqtt, DeviceId, DeviceType, Topic};

use bytes::Buf;
use futures_util::StreamExt;
use hyper::{Body, Request, Response};
use log::{debug, info, trace};
use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode, QOS_1};

use crate::web_service::{auth::validate_autorization, StatusCode};
use crate::{reporter, Result};

pub async fn query(request: Request<Body>) -> Result<Response<Body>> {
    validate_autorization(request, "devices_query", |request| async move {
        let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
        let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
        let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");

        let mut mqtt_client =
            connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "alisa_query")
                .await
                .expect("failed to connect mqtt");

        let request_id = String::from(std::str::from_utf8(
            request.headers().get("X-Request-Id").unwrap().as_bytes(),
        )?);

        let body = hyper::body::aggregate(request).await?;
        unsafe {
            trace!("[query]: {}", std::str::from_utf8_unchecked(body.chunk()));
        }

        let query: StateRequest = serde_json::from_slice(body.chunk())?;
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
        let response_topic = Topic::StateResponse(request_id.clone()).to_string();

        mqtt_client.subscribe(&response_topic, QOS_1);
        let mut stream = mqtt_client.get_stream(1);

        let mut props = Properties::new();
        props.push_string(PropertyCode::ResponseTopic, &response_topic)?;

        debug!("request to {}: {:?}", request_topic, request);
        debug!("waiting for response on {}", response_topic);

        let request_msg = MessageBuilder::new()
            .topic(request_topic)
            .properties(props)
            .payload(serde_json::to_vec(&request)?)
            .finalize();

        mqtt_client.publish(request_msg);

        let mut device_ids: HashSet<_> = HashSet::from_iter(request.device_ids);
        let mut devices = vec![];

        while let Ok(Some(msg_opt)) =
            tokio::time::timeout(Duration::from_secs(3), stream.next()).await
        {
            if let Some(msg) = msg_opt {
                debug!("msg: {:?}", msg);
                debug!("msg_str: {:?}", msg.payload_str());

                handle_message(msg, &mut device_ids, &mut devices);
            }

            if device_ids.is_empty() {
                break;
            }
        }

        mqtt_client.stop_stream();
        mqtt_client.unsubscribe(&response_topic);

        let response = StateResponse::new(request_id, devices);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}

fn handle_message(
    msg: Message,
    device_ids: &mut HashSet<DeviceId>,
    devices: &mut Vec<StateDevice>,
) {
    use transport::state::StateResponse;

    let response: StateResponse = serde_json::from_slice(msg.payload()).unwrap();
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
            DeviceType::TemperatureSensor | DeviceType::VacuumCleaner => (),
        },
    }
}
