mod types {
    pub mod device_id;
    pub mod device_type;
    pub mod room;
}

pub use types::device_id::DeviceId;
pub use types::device_type::DeviceType;
pub use types::room::Room;

mod inspinia_controller;
mod read_socket;
mod web_service;

pub use inspinia_controller::InspiniaController;
pub use read_socket::{read_from_socket, Handler as SocketHandler};
pub use web_service::web_handler;

mod state_manager;
pub use state_manager::StateManager;

mod update_state;
pub use update_state::update_devices_state;

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;
