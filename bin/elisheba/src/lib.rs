use sonoff::SonoffDevice;
use transport::{elisheba::State, Room};

use log::error;

pub struct Storage {
    nursery_light: Option<State>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            nursery_light: None,
        }
    }

    pub fn apply(&mut self, device: &SonoffDevice) -> Option<State> {
        let state = map_device_to_state(device)?;

        match state.room {
            Room::Nursery => {
                if let Some(ref nursery_light) = self.nursery_light {
                    if nursery_light.is_enabled == state.is_enabled {
                        return None;
                    }
                }

                self.nursery_light = Some(state);
                Some(state)
            }
            _ => None,
        }
    }
}

fn map_device_to_state(device: &SonoffDevice) -> Option<State> {
    let room = map_id_to_room(&device.id)?;
    let switch = device.meta["switches"][0]["switch"].as_str()?;
    let is_enabled = match switch {
        "on" => true,
        "off" => false,
        other => {
            error!("unknown switch state {other}");
            false
        }
    };

    Some(State { is_enabled, room })
}

fn map_id_to_room(id: &str) -> Option<Room> {
    match id {
        "10020750eb" => Some(Room::Nursery),
        _ => None,
    }
}
