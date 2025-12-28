mod local;
mod protocol;
mod util;
mod vacuum;

pub use vacuum::{CleanupMode, FanSpeed, State, Status, Vacuum};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
