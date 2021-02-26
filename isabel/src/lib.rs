mod device;
mod discover;
mod message;

mod vacuum;
pub use vacuum::{FanSpeed, Vacuum};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

pub type Token = [u8; 16];
