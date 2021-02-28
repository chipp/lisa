mod types {
    pub mod device_id;
    pub mod device_type;
    pub mod room;
}

pub use types::device_id::DeviceId;
pub use types::device_type::DeviceType;
pub use types::room::Room;

mod service;
mod socket_handler;

pub use service::service;
pub use socket_handler::SocketHandler;

mod state;
mod update_state;

pub use state::state_for_device;
pub use update_state::update_devices_state;

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;
