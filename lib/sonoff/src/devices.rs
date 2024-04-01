use std::collections::HashMap;
use std::net::SocketAddrV4;

#[derive(Debug, Clone)]
pub struct SonoffDevice {
    pub id: String,
    pub addr: SocketAddrV4,
    pub meta: serde_json::Value,
}

#[derive(Debug, Default)]
pub struct SonoffDevicesManager {
    pub devices: HashMap<String, SonoffDevice>,
}
