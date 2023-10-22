mod device;
mod discover;
mod message;
mod token;
mod vacuum;

pub use token::{parse_token, Token};
pub use vacuum::{FanSpeed, Vacuum};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
