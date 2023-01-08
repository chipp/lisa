mod device;
pub use device::{port_name::PortName, port_type::PortType, Device, Port, Properties, Room};

mod template;
pub use template::download_template;
use ws_client::OutMessage;

mod messages;
pub use messages::incoming::{ReceivedMessage, UpdateMessageContent};
pub use messages::outgoing::{RegisterMessage, SqlRequestMessage, UpdateStateMessage};

mod ws_client;
pub use ws_client::WSClient;

mod device_manager;
pub use device_manager::DeviceManager;

use serde::Deserialize;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

#[derive(Debug, Deserialize)]
pub struct PortState {
    pub id: String,
    pub value: Option<String>,
}
