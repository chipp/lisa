mod inspinia_client;
pub use inspinia_client::InspiniaClient;

mod types;
pub use types::DeviceType;

mod state_payload;
pub use state_payload::{Capability, StatePayload};

mod topic;
pub use topic::{set_topics_and_qos, Topic};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
