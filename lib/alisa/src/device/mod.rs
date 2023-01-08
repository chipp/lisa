pub mod port;
pub mod port_name;
pub mod port_type;
pub mod properties;
pub mod room;

pub use port::Port;
pub use properties::Properties;
pub use room::Room;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Device {
    pub id: String,
    pub room: Room,
    pub properties: Properties,
    pub ports: HashMap<String, Port>,
}
