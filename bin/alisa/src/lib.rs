mod types {
    mod capability;
    mod device_id;
    mod device_type;
    mod room;
    mod topic;
    mod update_payload;

    pub use capability::Capability;
    pub use device_id::DeviceId;
    pub use device_type::DeviceType;
    pub use room::Room;
    pub use topic::{state_topics_and_qos, Service, Topic};
    pub use update_payload::UpdatePayload;
}

mod reporter;
mod web_service;

pub use reporter::report_state;
pub use types::{state_topics_and_qos, Capability, DeviceId, DeviceType, Room, Service, Topic};
pub use web_service::web_handler;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
