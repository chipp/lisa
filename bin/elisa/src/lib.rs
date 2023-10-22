mod capability;
mod room;
mod topic;

pub use capability::Capability;
pub use room::Room;
pub use topic::{set_topics_and_qos, Topic};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
