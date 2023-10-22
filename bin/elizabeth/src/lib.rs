mod inspinia_client;
mod state_payload;
mod topic;

pub use inspinia_client::InspiniaClient;
pub use state_payload::StatePayload;
pub use topic::set_topics_and_qos;

pub use topics::{Device, ElizabethState as State};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
