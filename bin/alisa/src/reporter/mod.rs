mod recuperator;
mod thermostat;
mod vacuum_cleaner;

use recuperator::prepare_recuperator_update;
use thermostat::prepare_thermostat_update;

use crate::{reporter::vacuum_cleaner::prepare_vacuum_updates, Result};
use alice::StateResponse;
use topics::{Device, ElisaState, ElizabethState, Room};

use chrono::Utc;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error};
use serde_json::Value;

pub enum State {
    Elizabeth(ElizabethState),
    Elisa(ElisaState),
}

pub struct Event {
    pub device: Device,
    pub room: Option<Room>,
    pub state: State,
    pub payload: Value,
}

pub async fn report_state(event: Event) -> Result<()> {
    let mut devices = vec![];

    let Event {
        device,
        room,
        state,
        payload,
    } = event;

    match (device, state) {
        (Device::Recuperator, State::Elizabeth(state)) => {
            devices.push(prepare_recuperator_update(room, state, payload)?);
        }
        (Device::Thermostat, State::Elizabeth(state)) => {
            devices.push(prepare_thermostat_update(room, state, payload)?);
        }
        (Device::TemperatureSensor, _) => todo!(),
        (Device::VacuumCleaner, State::Elisa(state)) => {
            let mut result = prepare_vacuum_updates(state, payload)?;
            devices.append(&mut result);
        }
        _ => todo!(), // TODO: throw an error
    };

    let now = Utc::now();
    let body = StateResponse::notification_body(now.timestamp(), "chipp", devices);

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
