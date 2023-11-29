mod recuperator;
mod thermostat;
mod vacuum_cleaner;

pub use recuperator::{prepare_recuperator_current_state, prepare_recuperator_update};
pub use thermostat::{prepare_thermostat_current_state, prepare_thermostat_update};
pub use vacuum_cleaner::prepare_vacuum_updates;

use crate::Result;
use alice::{StateDevice, StateResponse};
use transport::elisa::State as ElisaState;
use transport::elizabeth::State as ElizabethState;
use transport::DeviceType;

use chrono::Utc;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error};

pub enum State {
    Elizabeth(ElizabethState),
    Elisa(ElisaState),
}

pub async fn report_state(state: State) -> Result<()> {
    let devices = match state {
        State::Elizabeth(state) => {
            vec![prepare_elizabeth_device(state)?]
        }
        State::Elisa(state) => prepare_vacuum_updates(state),
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
                let body = hyper::body::to_bytes(response.into_body()).await?;

                error!("{}", String::from_utf8(body.to_vec()).unwrap());
            }
        }
        Err(err) => error!("unable to report state changes {}", err),
    }

    Ok(())
}

fn prepare_elizabeth_device(state: ElizabethState) -> Result<StateDevice> {
    match state.device_type {
        DeviceType::Recuperator => prepare_recuperator_update(state),
        DeviceType::Thermostat => prepare_thermostat_update(state),
        DeviceType::TemperatureSensor | DeviceType::VacuumCleaner => todo!(),
    }
}