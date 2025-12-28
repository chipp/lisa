mod local;
mod protocol;
mod util;
mod vacuum;

pub use vacuum::{
    CleanupMode, DockErrorCode, ErrorCode, FanSpeed, State, Status, Vacuum, WashPhase, WashStatus,
};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
