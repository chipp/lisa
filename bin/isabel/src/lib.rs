mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

mod db;
pub use db::Db;
