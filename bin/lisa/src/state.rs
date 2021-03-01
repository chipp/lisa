use std::str::FromStr;

use alice::{Mode, ModeFunction, StateCapability, StateDevice, StateProperty, StateResponse};
use chrono::Utc;
use http_client::{parse_void, HttpClient};
use log::{error, info};

use crate::DeviceId;
use crate::DeviceType::*;
use crate::Room::{self, *};

pub struct StateManager {
    pub vacuum_state: VacuumState,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            vacuum_state: VacuumState::default(),
        }
    }

    pub async fn report_if_necessary(&mut self) {
        let mut devices = vec![];

        if self.vacuum_state.modified {
            for room in Room::all_rooms() {
                let device_id = DeviceId::vacuum_cleaner_at_room(*room);

                devices.push(StateDevice::new_with_properties_and_capabilities(
                    device_id.to_string(),
                    vec![StateProperty::battery_level(
                        self.vacuum_state.battery as f32,
                    )],
                    vec![
                        StateCapability::on_off(self.vacuum_state.is_enabled),
                        StateCapability::mode(
                            ModeFunction::WorkSpeed,
                            self.vacuum_state.work_speed,
                        ),
                    ],
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

        let body = request.body.as_ref().unwrap().clone();

        match client.perform_request(request, parse_void).await {
            Ok(_) => {
                info!("successfully notified alice about changes");
                self.vacuum_state.modified = false
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
            | (LivingRoom, VacuumCleaner) => {
                Some(StateDevice::new_with_properties_and_capabilities(
                    device_id.to_string(),
                    vec![StateProperty::battery_level(
                        self.vacuum_state.battery as f32,
                    )],
                    vec![
                        StateCapability::on_off(self.vacuum_state.is_enabled),
                        StateCapability::mode(
                            ModeFunction::WorkSpeed,
                            self.vacuum_state.work_speed,
                        ),
                    ],
                ))
            }
            _ => None,
        }
    }
}

#[derive(Default, PartialEq)]
pub struct VacuumState {
    is_enabled: bool,
    battery: u8,
    work_speed: Mode,

    modified: bool,
}

impl VacuumState {
    pub fn set_battery(&mut self, battery: u8) {
        if self.battery != battery {
            self.battery = battery;
            self.modified = true;
        }
    }

    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        if self.is_enabled != is_enabled {
            self.is_enabled = is_enabled;
            self.modified = true;
        }
    }

    pub fn set_work_speed(&mut self, work_speed: String) {
        let vacuum_work_speed = Mode::from_str(&work_speed).unwrap();
        if self.work_speed != vacuum_work_speed {
            self.work_speed = vacuum_work_speed;
            self.modified = true;
        }
    }
}
