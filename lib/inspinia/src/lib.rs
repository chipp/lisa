mod device;
pub use device::{Device, FanSpeed, Port, PortName, PortType, Properties, Room};

mod error;
pub use error::Error;

mod template;
pub use template::download_template;

mod messages;
pub use messages::incoming::{ReceivedMessage, UpdateMessageContent};
pub use messages::outgoing::{
    KeepAliveMessage, RegisterMessage, SqlRequestMessage, UpdateStateMessage,
};

mod ws_client;
use ws_client::OutgoingMessage;
pub use ws_client::WsClient;

mod device_manager;
pub use device_manager::DeviceManager;

use serde::Deserialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize)]
pub struct PortState {
    pub id: String,
    pub value: Option<String>,
}
