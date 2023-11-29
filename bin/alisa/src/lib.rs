mod types {
    mod action_payload;
    pub use action_payload::Action;
}

mod reporter;
mod web_service;

pub use reporter::{report_state, State};
pub use types::Action;
pub use web_service::web_handler;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
