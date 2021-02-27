mod device_id;
mod device_type;
mod room;
mod socket_handler;
mod state;
mod update_state;

pub use device_id::DeviceId;
pub use device_type::DeviceType;
pub use room::Room;

pub use state::state_for_device;
pub use update_state::update_devices_state;

pub use socket_handler::SocketHandler;

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;
