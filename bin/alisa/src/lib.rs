mod types {
    mod action_payload;
    mod device_id;
    mod topic;
    mod vacuum_fan_speed;

    pub use action_payload::{Action, ActionPayload};
    pub use device_id::DeviceId;
    pub use topic::{create_action_topic, state_topics_and_qos};
    pub use vacuum_fan_speed::FanSpeed as VacuumFanSpeed;
}

mod reporter;
mod web_service;

pub use reporter::{report_state, Event, State};
pub use types::{create_action_topic, state_topics_and_qos, Action, ActionPayload, DeviceId};
pub use web_service::web_handler;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
