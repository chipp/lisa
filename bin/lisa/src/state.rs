mod reportable_property;

mod vacuum_state;
pub use vacuum_state::VacuumState;

mod sensor_state;
pub use sensor_state::SensorState;

mod thermostat_state;
pub use thermostat_state::ThermostatState;

mod recuperator_state;
pub use recuperator_state::RecuperatorState;

use alice::{StateCapability, StateDevice, StateProperty, StateResponse};
use chrono::Utc;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error};

use crate::DeviceId;
use crate::Room;
use crate::Room::*;

trait State: Send {
    fn prepare_updates(&self, devices: &mut Vec<StateDevice>);
    fn properties(&self, only_updated: bool) -> Vec<StateProperty>;
    fn capabilities(&self, only_updated: bool) -> Vec<StateCapability>;
    fn reset_modified(&mut self);
}

pub struct StateManager {
    pub vacuum: VacuumState,

    pub bedroom_sensor: SensorState,
    pub kitchen_sensor: SensorState,
    pub home_office_sensor: SensorState,

    thermostats: [ThermostatState; 4],
    recuperator: RecuperatorState,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            vacuum: VacuumState::default(),
            bedroom_sensor: SensorState::new(Bedroom),
            home_office_sensor: SensorState::new(HomeOffice),
            kitchen_sensor: SensorState::new(Kitchen),
            thermostats: [
                ThermostatState::new(Bedroom),
                ThermostatState::new(Nursery),
                ThermostatState::new(HomeOffice),
                ThermostatState::new(LivingRoom),
            ],
            recuperator: RecuperatorState::new(),
        }
    }

    pub fn thermostat_state_in_room(&mut self, room: Room) -> Option<&mut ThermostatState> {
        for state in self.thermostats.iter_mut() {
            if room == state.room() {
                return Some(state);
            }
        }

        None
    }

    pub fn recuperator_state(&mut self) -> &mut RecuperatorState {
        &mut self.recuperator
    }

    pub async fn report_if_necessary(&mut self) {
        let mut states: Vec<&mut dyn State> = vec![
            &mut self.vacuum,
            &mut self.bedroom_sensor,
            &mut self.home_office_sensor,
            &mut self.kitchen_sensor,
            &mut self.recuperator,
        ];

        states.extend(self.thermostats.iter_mut().map(|s| s as &mut dyn State));

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
