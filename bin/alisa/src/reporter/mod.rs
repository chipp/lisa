mod recuperator;
mod thermostat;

use recuperator::prepare_recuperator_update;
use thermostat::prepare_thermostat_update;

use crate::{DeviceType, Result, Topic};

use alice::StateResponse;

use chrono::Utc;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error};

pub async fn report_state(topic: Topic, payload: &[u8]) -> Result<()> {
    let Topic {
        service: _,
        topic_type: _,
        room,
        device_type,
        capability,
    } = topic;

    let device = match device_type {
        DeviceType::Recuperator => prepare_recuperator_update(room, capability, payload)?,
        DeviceType::Thermostat => prepare_thermostat_update(room, capability, payload)?,
        DeviceType::TemperatureSensor => todo!(),
        DeviceType::VacuumCleaner => todo!(),
    };

    let now = Utc::now();
    let body = StateResponse::notification_body(now.timestamp(), "chipp", vec![device]);

    debug!(
        "state update: {}",
        serde_json::to_string_pretty(&body).unwrap()
    );

    let skill_id = std::env::var("ALICE_SKILL_ID").expect("skill id is required");
    let token = std::env::var("ALICE_TOKEN").expect("token is required");

    let https = HttpsConnector::new();
    let client = Client::builder().build(https);

    let body = serde_json::to_vec(&body).unwrap();

    let request = Request::builder()
        .method(Method::POST)
        .uri(format!(
            "https://dialogs.yandex.net/api/v1/skills/{}/callback/state",
            skill_id
        ))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("OAuth {}", token))
        .body(Body::from(body))
        .unwrap();

    match client.request(request).await {
        Ok(response) => {
            if let StatusCode::ACCEPTED = response.status() {
                debug!("successfully notified alice about changes");
            } else {
                error!("unable to report state changes {}", response.status());
                error!("{:#?}", response);
            }
        }
        Err(err) => error!("unable to report state changes {}", err),
    }

    Ok(())
}
