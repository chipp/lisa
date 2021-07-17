mod command;
mod status;

use log::info;
pub use status::FanSpeed;
pub use status::Status;

use command::Command::{self, *};
use command::Mode::*;

use status::BinType;
use status::CleanMode;

use crate::device::Device;
use crate::Result;
use elisheba::Token;

use serde_json::from_value;
use serde_json::Value;

pub struct Vacuum {
    device: Device,
    last_cleaning_rooms: Vec<u8>,
}

impl Vacuum {
    pub fn new(ip: [u8; 4], token: Token<16>) -> Vacuum {
        Vacuum {
            device: Device::new(ip, token),
            last_cleaning_rooms: vec![],
        }
    }

    pub async fn status(&mut self) -> Result<Status> {
        let response = self.execute_command(GetProperties(status::FIELDS)).await?;
        let status = from_value(response)?;
        Ok(status)
    }

    pub async fn set_fan_speed(&mut self, fan_speed: FanSpeed) -> Result<()> {
        self.execute_command(SetFanSpeed(fan_speed)).await?;
        Ok(())
    }

    pub async fn set_clean_mode(&mut self, clean_mode: CleanMode) -> Result<()> {
        self.execute_command(SetCleanMode(clean_mode)).await?;
        Ok(())
    }

    pub async fn start(&mut self, room_ids: Vec<u8>) -> Result<()> {
        let status = self.status().await?;

        info!("{} and {}", status.bin_type, status.clean_mode);

        match (status.bin_type, status.clean_mode) {
            (BinType::NoBin | BinType::Water, _) => todo!(),
            (BinType::Vacuum, CleanMode::Vacuum) => {
                info!("don't change clean mode");
            }
            (BinType::VacuumAndWater, CleanMode::VacuumAndMop) => {
                info!("vacuum and water bin and vacuum and mop clean mode, do nothing")
            }
            (BinType::Vacuum, _) => self.set_clean_mode(CleanMode::Vacuum).await?,
            (BinType::VacuumAndWater, _) => self.set_clean_mode(CleanMode::VacuumAndMop).await?,
        };

        self.last_cleaning_rooms = room_ids.clone();

        self.execute_command(SetModeWithRooms(Start, &room_ids))
            .await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.execute_command(SetMode(Stop)).await?;
        Ok(())
    }

    pub async fn go_home(&mut self) -> Result<()> {
        self.execute_command(SetCharge).await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        let room_ids = self.last_cleaning_rooms.clone();

        self.execute_command(SetModeWithRooms(Pause, &room_ids))
            .await?;
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        let room_ids = self.last_cleaning_rooms.clone();

        self.execute_command(SetModeWithRooms(Start, &room_ids))
            .await?;
        Ok(())
    }

    async fn execute_command<'a>(&mut self, command: Command<'a>) -> Result<Value> {
        self.device.send(command.name(), command).await
    }
}
