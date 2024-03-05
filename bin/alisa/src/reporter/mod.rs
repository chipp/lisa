mod recuperator;
mod temperature_sensor;
mod thermostat;
mod vacuum_cleaner;

pub use recuperator::{prepare_recuperator_current_state, prepare_recuperator_update};
pub use temperature_sensor::prepare_sensor_update;
pub use thermostat::{prepare_thermostat_current_state, prepare_thermostat_update};
pub use vacuum_cleaner::prepare_vacuum_updates;

use crate::Result;
use alice::{StateDevice, StateResponse};
use transport::elizabeth::State as ElizabethState;
use transport::state::StateUpdate;
use transport::DeviceType;

use chrono::Utc;
use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error};

pub struct Reporter {
    inner: Client<HttpsConnector<HttpConnector>>,
    skill_id: String,
    token: String,
}

impl Reporter {
    pub fn new(skill_id: String, token: String) -> Self {
        let https = HttpsConnector::new();
        let inner = Client::builder().build(https);

        Self {
            inner,
            skill_id,
            token,
        }
    }

    pub async fn report_update(&self, update: StateUpdate) -> Result<()> {
        let devices = if let Some(devices) = device_from_update(update) {
            devices
        } else {
            return Ok(());
        };

        let now = Utc::now();
        let body = StateResponse::notification_body(now.timestamp(), "chipp", devices);

        debug!(
            "state update: {}",
            serde_json::to_string_pretty(&body).unwrap()
        );

        let body = serde_json::to_vec(&body).unwrap();

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!(
                "https://dialogs.yandex.net/api/v1/skills/{}/callback/state",
                self.skill_id
            ))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("OAuth {}", self.token))
            .body(Body::from(body))
            .unwrap();

        debug!("request: {:?}", request);

        match self.inner.request(request).await {
            Ok(response) => {
                if let StatusCode::ACCEPTED = response.status() {
                    debug!("successfully notified alice about changes");
                } else {
                    error!("unable to report state changes {}", response.status());
                    debug!("{:#?}", response);

                    let body = response.into_body().collect().await?.to_bytes();
                    debug!("{}", String::from_utf8(body.to_vec()).unwrap());
                }
            }
            Err(err) => error!("unable to report state changes {}", err),
        }

        Ok(())
    }
}

fn device_from_update(update: StateUpdate) -> Option<Vec<StateDevice>> {
    match update {
        StateUpdate::Elizabeth(state) => Some(vec![prepare_elizabeth_device(state)?]),
        StateUpdate::Elisa(state) => Some(prepare_vacuum_updates(state)),
        StateUpdate::Isabel(state) => Some(vec![prepare_sensor_update(state)]),
    }
}

fn prepare_elizabeth_device(state: ElizabethState) -> Option<StateDevice> {
    match state.device_type {
        DeviceType::Recuperator => prepare_recuperator_update(state),
        DeviceType::Thermostat => prepare_thermostat_update(state),
        DeviceType::TemperatureSensor | DeviceType::VacuumCleaner => todo!(),
    }
}
