use super::{port_name::PortName, port_type::PortType};

#[derive(Debug)]
pub struct Port {
    pub r#type: PortType,
    pub name: PortName,
}
