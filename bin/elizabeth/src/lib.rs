mod client;
pub use client::Client;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
