mod reporter;
mod web_service;

pub use reporter::Reporter;
pub use web_service::router;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
