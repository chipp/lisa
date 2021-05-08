mod vacuum_state;
use vacuum_state::VacuumState;

mod sensor_state;
use sensor_state::SensorState;

use alice::{StateDevice, StateResponse};
use chrono::Utc;
use http_client::{parse_void, HttpClient};
use log::{debug, error};

use crate::DeviceId;
use crate::DeviceType::*;
use crate::Room::{self, *};

pub struct StateManager {
    pub vacuum_state: VacuumState,
    pub nursery_sensor_state: SensorState,
    pub bedroom_sensor_state: SensorState,
    pub living_room_sensor_state: SensorState,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            vacuum_state: VacuumState::default(),
            nursery_sensor_state: SensorState::default(),
            bedroom_sensor_state: SensorState::default(),
            living_room_sensor_state: SensorState::default(),
        }
    }

    pub async fn report_if_necessary(&mut self) {
        let mut devices = vec![];

        {
            let properties = self.vacuum_state.properties(true);
            let capabilities = self.vacuum_state.capabilities(true);

            if !properties.is_empty() && !capabilities.is_empty() {
                for room in Room::all_rooms() {
                    let device_id = DeviceId::vacuum_cleaner_at_room(*room);

                    devices.push(StateDevice::new_with_properties_and_capabilities(
                        device_id.to_string(),
                        properties.clone(),
                        capabilities.clone(),
                    ));
                }
            }
        }

        {
            let properties = self.nursery_sensor_state.properties(true);

            if !properties.is_empty() {
                let device_id = DeviceId::temperature_sensor_at_room(Room::Nursery);

                devices.push(StateDevice::new_with_properties(
                    device_id.to_string(),
                    properties,
                ));
            }
        }

        {
            let properties = self.bedroom_sensor_state.properties(true);

            if !properties.is_empty() {
                let device_id = DeviceId::temperature_sensor_at_room(Room::Bedroom);

                devices.push(StateDevice::new_with_properties(
                    device_id.to_string(),
                    properties,
                ));
            }
        }

        {
            let properties = self.living_room_sensor_state.properties(true);

            if !properties.is_empty() {
                let device_id = DeviceId::temperature_sensor_at_room(Room::LivingRoom);

                devices.push(StateDevice::new_with_properties(
                    device_id.to_string(),
                    properties,
                ));
            }
        }

        if devices.is_empty() {
            return;
        }

        let now = Utc::now();
        let body = StateResponse::notification_body(now.timestamp(), "chipp", devices);

        let skill_id = std::env::var("ALICE_SKILL_ID").expect("skill id is required");
        let token = std::env::var("ALICE_TOKEN").expect("token is required");

        let client = HttpClient::new("https://dialogs.yandex.net/api/v1/skills").unwrap();

        let mut request = client.new_request(&[&skill_id, "callback", "state"]);
        request.set_method(http_client::HttpMethod::Post);
        request.set_json_body(&body);
        request.add_header("Authorization", format!("OAuth {}", token));
        request.set_retry_count(3);

        let body = request.body.as_ref().unwrap().clone();

        match client.perform_request(request, parse_void).await {
            Ok(_) => {
                debug!("successfully notified alice about changes");
                self.vacuum_state.reset_modified();
                self.nursery_sensor_state.reset_modified();
                self.bedroom_sensor_state.reset_modified();
                self.living_room_sensor_state.reset_modified();
            }
            Err(err) => {
                error!("unable to report state changes {}", err);
                error!("{}", std::str::from_utf8(&body).unwrap());
            }
        }
    }

    pub fn state_for_device(&self, device_id: DeviceId) -> Option<StateDevice> {
        let DeviceId { room, device_type } = &device_id;
        match (room, device_type) {
            (Hallway, VacuumCleaner)
            | (Corridor, VacuumCleaner)
            | (Bathroom, VacuumCleaner)
            | (Nursery, VacuumCleaner)
            | (Bedroom, VacuumCleaner)
            | (Kitchen, VacuumCleaner)
            | (Balcony, VacuumCleaner)
            | (LivingRoom, VacuumCleaner) => {
                Some(StateDevice::new_with_properties_and_capabilities(
                    device_id.to_string(),
                    self.vacuum_state.properties(false),
                    self.vacuum_state.capabilities(false),
                ))
            }
            (Nursery, TemperatureSensor) => Some(StateDevice::new_with_properties(
                device_id.to_string(),
                self.nursery_sensor_state.properties(false),
            )),
            (Bedroom, TemperatureSensor) => Some(StateDevice::new_with_properties(
                device_id.to_string(),
                self.bedroom_sensor_state.properties(false),
            )),
            (LivingRoom, TemperatureSensor) => Some(StateDevice::new_with_properties(
                device_id.to_string(),
                self.living_room_sensor_state.properties(false),
            )),
            _ => None,
        }
    }
}
