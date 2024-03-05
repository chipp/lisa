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
use log::{debug, error};
use reqwest::{header, Client, StatusCode};

pub struct Reporter {
    inner: Client,
    skill_id: String,
    token: String,
}

impl Reporter {
    pub fn new(skill_id: String, token: String) -> Self {
        let inner = Client::new();

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

        let result = self
            .inner
            .post(format!(
                "https://dialogs.yandex.net/api/v1/skills/{}/callback/state",
                self.skill_id
            ))
            .json(&body)
            .header(header::AUTHORIZATION, format!("OAuth {}", self.token))
            .send()
            .await;

        match result {
            Ok(response) => {
                if let StatusCode::ACCEPTED = response.status() {
                    debug!("successfully notified alice about changes");
                } else {
                    error!("unable to report state changes {}", response.status());
                    debug!("{:#?}", response);

                    let json: String = response.json().await?;
                    debug!("{}", json);
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
