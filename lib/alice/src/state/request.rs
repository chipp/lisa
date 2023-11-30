use serde::Deserialize;
use transport::DeviceId;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub id: DeviceId,
}
