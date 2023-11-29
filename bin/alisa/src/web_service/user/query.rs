use std::collections::HashSet;
use std::time::Duration;

use alice::{StateRequest, StateResponse};
use transport::{DeviceType, ResponseState};

use bytes::Buf;
use futures_util::StreamExt;
use hyper::{Body, Request, Response};
use log::trace;
use paho_mqtt::{MessageBuilder, Properties, QOS_1};

use crate::web_service::{auth::validate_autorization, StatusCode};
use crate::{reporter, Result};

pub async fn query(
    request: Request<Body>,
    mut mqtt_client: paho_mqtt::AsyncClient,
) -> Result<Response<Body>> {
    validate_autorization(request, "devices_query", |request| async move {
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

        let request_topic = "request";
        let response_topic = format!("response/{request_id}");

        mqtt_client.subscribe(&response_topic, QOS_1);
        let mut stream = mqtt_client.get_stream(1);

        let mut props = Properties::new();
        props.push_string(paho_mqtt::PropertyCode::ResponseTopic, &response_topic)?;

        let request = MessageBuilder::new()
            .topic(request_topic)
            .properties(props)
            .payload(serde_json::to_vec(&device_ids)?)
            .finalize();

        mqtt_client.publish(request);

        let mut device_ids: HashSet<_> = HashSet::from_iter(device_ids);
        let mut devices = vec![];

        while let Ok(Some(msg_opt)) =
            tokio::time::timeout(Duration::from_secs(10), stream.next()).await
        {
            if let Some(msg) = msg_opt {
                let response: ResponseState = serde_json::from_slice(msg.payload()).unwrap();

                match response {
                    ResponseState::Elisa(state) => {
                        let states = reporter::prepare_vacuum_updates(state);

                        for state in states {
                            if device_ids.contains(&state.id()) {
                                device_ids.remove(&state.id());
                                devices.push(state);
                            }
                        }
                    }
                    ResponseState::Elizabeth(state) => match state.device_type {
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

            if device_ids.is_empty() {
                break;
            }
        }

        let response = StateResponse::new(request_id, devices);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}
