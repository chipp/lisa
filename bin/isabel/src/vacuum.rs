mod status;
pub use status::Status;

pub use status::FanSpeed;

use crate::device::Device;
use crate::{Result, Token};

use serde_json::{from_value, json};

pub struct Vacuum {
    device: Device,
}

impl Vacuum {
    pub fn new(ip: [u8; 4], token: Token) -> Vacuum {
        Vacuum {
            device: Device::new(ip, token),
        }
    }

    pub async fn status(&mut self) -> Result<Status> {
        let response = self.device.send("get_prop", json!(status::FIELDS)).await?;
        let status = from_value(response)?;
        Ok(status)
    }

    pub async fn set_fan_speed(&mut self, fan_speed: FanSpeed) -> Result<()> {
        self.device.send("set_suction", json!([fan_speed])).await?;
        Ok(())
    }

    pub async fn start(&mut self, room_ids: Vec<u8>) -> Result<()> {
        let mut room_ids = room_ids;
        let mut params = vec![0, 1, room_ids.len() as u8];
        params.append(&mut room_ids);

        self.device.send("set_mode_withroom", json!(params)).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.device.send("set_mode", json!([0, 0])).await?;
        Ok(())
    }

    pub async fn go_home(&mut self) -> Result<()> {
        self.device.send("set_charge", json!([1])).await?;
        Ok(())
    }
}