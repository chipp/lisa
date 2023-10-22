mod action;
mod capability;
mod room;
mod status;
mod topic;

pub use action::Action;
pub use capability::Capability;
pub use room::Room;
pub use status::Status;
pub use topic::{actions_topics_and_qos, Topic};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
