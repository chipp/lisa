mod types {
    mod action_payload;
    mod device_id;

    pub use action_payload::Action;
    pub use device_id::DeviceId;
}

mod reporter;
mod web_service;

pub use reporter::{report_state, State};
pub use types::{Action, DeviceId};
pub use web_service::web_handler;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
