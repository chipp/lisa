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
pub use state::{StateManager, ThermostatState};

mod update_state;
pub use update_state::update_devices_state;

mod handle_socket_client;
pub use handle_socket_client::read_from_socket;

#[cfg(feature = "inspinia")]
mod inspinia_controller;

#[cfg(feature = "inspinia")]
pub use inspinia_controller::InspiniaController;

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;
