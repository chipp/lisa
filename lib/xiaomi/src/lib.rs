mod device;
mod discover;
mod message;
mod vacuum;

mod error;
pub use error::Error;

pub use vacuum::{FanSpeed, Status, Vacuum};

pub type Result<T> = std::result::Result<T, Error>;
