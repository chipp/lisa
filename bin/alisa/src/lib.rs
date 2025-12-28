mod reporter;
mod web_service;

pub use reporter::Reporter;
pub use web_service::router;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
