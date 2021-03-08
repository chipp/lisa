mod device;
mod discover;
mod message;
mod socket_handler;
mod token;
mod vacuum;
mod vacuum_controller;

pub use socket_handler::SocketHandler;
pub use vacuum::{FanSpeed, Vacuum};
pub use vacuum_controller::VacuumController;

pub use token::{parse_token, Token};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;
