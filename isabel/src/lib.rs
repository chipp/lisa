mod discover;
mod message;

mod device;
pub use device::Device;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

pub type Token = [u8; 16];
