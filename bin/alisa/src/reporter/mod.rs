mod light;
mod recuperator;
mod temperature_sensor;
mod thermostat;
mod vacuum_cleaner;

pub use light::prepare_light_update;
pub use recuperator::{prepare_recuperator_current_state, prepare_recuperator_update};
pub use temperature_sensor::prepare_sensor_update;
pub use thermostat::{prepare_thermostat_current_state, prepare_thermostat_update};
pub use vacuum_cleaner::prepare_vacuum_updates;

use crate::Result;
use alice::{StateDevice, StateResponse};
use transport::elizabeth::State as ElizabethState;
use transport::state::StateUpdate;
use transport::DeviceType;

use chipp_http::{HttpClient, HttpMethod, NoInterceptor};
use chrono::Utc;
use log::{debug, error};

pub struct Reporter {
    inner: HttpClient<NoInterceptor>,
    skill_id: String,
    token: String,
}

impl Reporter {
    pub fn new(skill_id: String, token: String) -> Self {
        let inner = HttpClient::new("https://dialogs.yandex.net/api/v1").unwrap();

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

        let mut request = self
            .inner
            .new_request(["skills", &self.skill_id, "callback", "state"]);

        request.method = HttpMethod::Post;
        request.set_json_body(&body);
        request.add_header("Authorization", format!("OAuth {}", self.token));

        let result = self
            .inner
            .perform_request(request, |req, res| {
                if res.status_code == 202 {
                    Ok(res)
                } else {
                    Err((req, res).into())
                }
            })
            .await;

        match result {
            Ok(_) => {
                debug!("successfully notified alice about changes");
            }
            Err(err) => {
                if let chipp_http::ErrorKind::HttpError(ref res) = err.kind {
                    error!("unable to report state changes {}", res.status_code);
                    debug!("{:#?}", res);

                    if let Ok(json) = std::str::from_utf8(&res.body) {
                        debug!("{}", json);
                    } else {
                        error!("unable to report state changes {}", err)
                    }
                } else {
                    error!("unable to report state changes {}", err)
                }
            }
        }

        Ok(())
    }
}

fn device_from_update(update: StateUpdate) -> Option<Vec<StateDevice>> {
    match update {
        StateUpdate::Elizabeth(state) => Some(vec![prepare_elizabeth_device(state)?]),
        StateUpdate::Elisa(state) => Some(prepare_vacuum_updates(state)),
        StateUpdate::Isabel(state) => Some(vec![prepare_sensor_update(state)]),
        StateUpdate::Elisheba(state) => Some(vec![prepare_light_update(state)]),
    }
}

fn prepare_elizabeth_device(state: ElizabethState) -> Option<StateDevice> {
    match state.device_type {
        DeviceType::Recuperator => prepare_recuperator_update(state),
        DeviceType::Thermostat => prepare_thermostat_update(state),
        _ => unreachable!(),
    }
}
