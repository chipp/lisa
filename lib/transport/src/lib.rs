pub mod elisa;
pub mod elizabeth;

pub mod action {
    mod request;
    mod response;

    pub use request::Action;
    pub use request::Request as ActionRequest;
    pub use response::ActionResult;
    pub use response::Response as ActionResponse;
}

pub mod state {
    mod request;
    mod response;
    mod update;

    pub use request::Request as StateRequest;
    pub use response::Response as StateResponse;
    pub use update::Update as StateUpdate;
}

mod device_id;
pub use device_id::DeviceId;

mod topic;
pub use topic::Topic;

use serde::{Deserialize, Serialize};
use str_derive::Str;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Recuperator,
    TemperatureSensor,
    Thermostat,
    VacuumCleaner,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Bathroom,
    Bedroom,
    Corridor,
    Hallway,
    HomeOffice,
    Kitchen,
    LivingRoom,
    Nursery,
    Toilet,
}

impl Room {
    pub const fn all_rooms() -> [Room; 9] {
        [
            Room::Bathroom,
            Room::Bedroom,
            Room::Corridor,
            Room::Hallway,
            Room::HomeOffice,
            Room::Kitchen,
            Room::LivingRoom,
            Room::Nursery,
            Room::Toilet,
        ]
    }
}
