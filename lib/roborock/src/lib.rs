mod local;
mod protocol;
mod util;
mod vacuum;

mod error;
pub use error::Error;

pub use vacuum::{
    CleanupMode, DockErrorCode, ErrorCode, FanSpeed, State, Status, Vacuum, WashPhase, WashStatus,
};

pub type Result<T> = std::result::Result<T, Error>;
