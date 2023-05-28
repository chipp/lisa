mod device;
pub use device::{Device, FanSpeed, Port, PortName, PortType, Properties, Room};

mod template;
pub use template::download_template;

mod messages;
pub use messages::incoming::{ReceivedMessage, UpdateMessageContent};
pub use messages::outgoing::{
    KeepAliveMessage, RegisterMessage, SqlRequestMessage, UpdateStateMessage,
};

mod ws_client;
use ws_client::OutgoingMessage;
pub use ws_client::{WsClient, WsError};

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
