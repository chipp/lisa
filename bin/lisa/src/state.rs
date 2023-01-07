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
use crate::DeviceType::*;
use crate::Room::*;

trait State {
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
        let mut devices = vec![];

        self.vacuum_state.prepare_updates(&mut devices);

        self.bedroom_sensor_state.prepare_updates(&mut devices);
        self.home_office_sensor_state.prepare_updates(&mut devices);
        self.kitchen_sensor_state.prepare_updates(&mut devices);

        self.bedroom_thermostat_state.prepare_updates(&mut devices);
        self.nursery_thermostat_state.prepare_updates(&mut devices);
        self.home_office_thermostat_state
            .prepare_updates(&mut devices);
        self.living_room_thermostat_state
            .prepare_updates(&mut devices);

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
                    self.vacuum_state.reset_modified();
                    self.bedroom_sensor_state.reset_modified();
                    self.home_office_sensor_state.reset_modified();
                    self.kitchen_sensor_state.reset_modified();
                } else {
                    error!("unable to report state changes {}", response.status());
                    error!("{:#?}", response);
                }
            }
            Err(err) => error!("unable to report state changes {}", err),
        }
    }

    pub fn state_for_device(&self, device_id: DeviceId) -> Option<StateDevice> {
        let DeviceId { room, device_type } = &device_id;
        match (room, device_type) {
            (Bathroom, VacuumCleaner)
            | (Bedroom, VacuumCleaner)
            | (Corridor, VacuumCleaner)
            | (Hallway, VacuumCleaner)
            | (HomeOffice, VacuumCleaner)
            | (Kitchen, VacuumCleaner)
            | (LivingRoom, VacuumCleaner)
            | (Nursery, VacuumCleaner)
            | (Toilet, VacuumCleaner) => Some(StateDevice::new_with_properties_and_capabilities(
                device_id.to_string(),
                self.vacuum_state.properties(false),
                self.vacuum_state.capabilities(false),
            )),
            (Nursery, TemperatureSensor) => Some(StateDevice::new_with_properties(
                device_id.to_string(),
                self.bedroom_sensor_state.properties(false),
            )),
            (Bedroom, TemperatureSensor) => Some(StateDevice::new_with_properties(
                device_id.to_string(),
                self.home_office_sensor_state.properties(false),
            )),
            (LivingRoom, TemperatureSensor) => Some(StateDevice::new_with_properties(
                device_id.to_string(),
                self.kitchen_sensor_state.properties(false),
            )),
            _ => None,
        }
    }
}
