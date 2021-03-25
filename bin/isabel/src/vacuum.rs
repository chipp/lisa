mod status;
pub use status::Status;

pub use status::FanSpeed;

use crate::device::Device;
use crate::Result;
use elisheba::Token16;

use serde_json::{from_value, json};

pub struct Vacuum {
    device: Device,
    last_cleaning_rooms: Vec<u8>,
}

impl Vacuum {
    pub fn new(ip: [u8; 4], token: Token16) -> Vacuum {
        Vacuum {
            device: Device::new(ip, token),
            last_cleaning_rooms: vec![],
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
        self.last_cleaning_rooms = room_ids.clone();

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

    pub async fn pause(&mut self) -> Result<()> {
        let mut room_ids = self.last_cleaning_rooms.clone();
        let mut params = vec![0, 2, room_ids.len() as u8];
        params.append(&mut room_ids);

        self.device.send("set_mode_withroom", json!(params)).await?;
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        let mut room_ids = self.last_cleaning_rooms.clone();
        let mut params = vec![0, 1, room_ids.len() as u8];
        params.append(&mut room_ids);

        self.device.send("set_mode_withroom", json!(params)).await?;
        Ok(())
    }
}
