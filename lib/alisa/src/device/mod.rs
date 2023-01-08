mod fan_speed;
mod port;
mod port_name;
mod port_type;
mod properties;
mod room;

pub use fan_speed::FanSpeed;
pub use port::Port;
pub use port_name::PortName;
pub use port_type::PortType;
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
