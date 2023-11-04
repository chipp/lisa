mod inspinia_client;
pub use inspinia_client::InspiniaClient;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
