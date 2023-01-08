mod vacuum_state;
pub use vacuum_state::VacuumState;

mod sensor_state;
pub use sensor_state::SensorState;

mod thermostat_state;
pub use thermostat_state::ThermostatState;

use alice::{StateCapability, StateDevice, StateProperty, StateResponse};
use chrono::Utc;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error};

use crate::DeviceId;
use crate::Room::*;

trait State: Send {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>);
    fn properties(&self, only_updated: bool) -> Vec<StateProperty>;
    fn capabilities(&self, only_updated: bool) -> Vec<StateCapability>;
    fn reset_modified(&mut self);
}

pub struct StateManager {
    pub vacuum_state: VacuumState,

    pub bedroom_sensor_state: SensorState,
    pub kitchen_sensor_state: SensorState,
    pub home_office_sensor_state: SensorState,

    pub bedroom_thermostat_state: ThermostatState,
    pub living_room_thermostat_state: ThermostatState,
    pub nursery_thermostat_state: ThermostatState,
    pub home_office_thermostat_state: ThermostatState,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            vacuum_state: VacuumState::default(),

            bedroom_sensor_state: SensorState::new(Bedroom),
            home_office_sensor_state: SensorState::new(HomeOffice),
            kitchen_sensor_state: SensorState::new(Kitchen),

            bedroom_thermostat_state: ThermostatState::new(Bedroom),
            nursery_thermostat_state: ThermostatState::new(Nursery),
            home_office_thermostat_state: ThermostatState::new(HomeOffice),
            living_room_thermostat_state: ThermostatState::new(LivingRoom),
        }
    }

    pub async fn report_if_necessary(&mut self) {
        let mut states: Vec<&mut dyn State> = vec![
            &mut self.vacuum_state,
            &mut self.bedroom_sensor_state,
            &mut self.home_office_sensor_state,
            &mut self.kitchen_sensor_state,
        ];

        if cfg!(feature = "inspinia") {
            states.push(&mut self.bedroom_thermostat_state);
            states.push(&mut self.nursery_thermostat_state);
            states.push(&mut self.home_office_thermostat_state);
            states.push(&mut self.living_room_thermostat_state);
        }

        let mut devices = vec![];

        for state in &states {
            state.prepare_updates(&mut devices);
        }

        if devices.is_empty() {
            return;
        }

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

                    for state in states {
                        state.reset_modified();
                    }
                } else {
                    error!("unable to report state changes {}", response.status());
                    error!("{:#?}", response);
                }
            }
            Err(err) => error!("unable to report state changes {}", err),
        }
    }

    pub fn state_for_device(&self, _device_id: DeviceId) -> Option<StateDevice> {
        None
    }
}
