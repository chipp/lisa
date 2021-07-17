mod command;
mod command_executor;
mod status;

pub use status::FanSpeed;
pub use status::Status;
pub use status::WaterGrade;

use command::Command::{self, *};
use command::Mode::*;
use command_executor::{CommandExecutor, CommandExecutorTrait};

use status::BinType;
use status::CleanMode;

use crate::device::Device;
use crate::Result;
use elisheba::Token;

use log::info;
use serde_json::from_value;

pub struct Vacuum {
    executor: Box<dyn CommandExecutorTrait + Send>,
    last_cleaning_rooms: Vec<u8>,
}

impl Vacuum {
    pub fn new(ip: [u8; 4], token: Token<16>) -> Vacuum {
        Vacuum {
            executor: Box::new(CommandExecutor::new(Device::new(ip, token))),
            last_cleaning_rooms: vec![],
        }
    }

    pub async fn status(&mut self) -> Result<Status> {
        let response = self
            .executor
            .execute_command(GetProperties(status::FIELDS))
            .await?;
        let status = from_value(response)?;
        Ok(status)
    }

    pub async fn set_fan_speed(&mut self, fan_speed: FanSpeed) -> Result<()> {
        self.executor
            .execute_command(SetFanSpeed(fan_speed))
            .await?;
        Ok(())
    }

    pub async fn set_water_grade(&mut self, water_grade: WaterGrade) -> Result<()> {
        self.executor
            .execute_command(SetWaterGrade(water_grade))
            .await?;
        Ok(())
    }

    pub async fn set_clean_mode(&mut self, clean_mode: CleanMode) -> Result<()> {
        self.executor
            .execute_command(SetCleanMode(clean_mode))
            .await?;
        Ok(())
    }

    pub async fn start(&mut self, room_ids: Vec<u8>) -> Result<()> {
        let status = self.status().await?;

        info!("{} and {}", status.bin_type, status.clean_mode);

        match (status.bin_type, status.clean_mode) {
            (BinType::NoBin | BinType::Water, _) => todo!(),
            (BinType::Vacuum, CleanMode::Vacuum)
            | (BinType::VacuumAndWater, CleanMode::VacuumAndMop) => {
                info!("don't change clean mode");
            }
            (BinType::Vacuum, _) => self.set_clean_mode(CleanMode::Vacuum).await?,
            (BinType::VacuumAndWater, _) => {
                self.set_clean_mode(CleanMode::VacuumAndMop).await?;

                if status.water_grade != WaterGrade::High {
                    self.set_water_grade(WaterGrade::High).await?
                }
            }
        };

        self.last_cleaning_rooms = room_ids.clone();

        self.executor
            .execute_command(SetModeWithRooms(Start, room_ids))
            .await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.executor.execute_command(SetMode(Stop)).await?;
        Ok(())
    }

    pub async fn go_home(&mut self) -> Result<()> {
        self.executor.execute_command(SetCharge).await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        let room_ids = self.last_cleaning_rooms.clone();

        self.executor
            .execute_command(SetModeWithRooms(Pause, room_ids))
            .await?;
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        let room_ids = self.last_cleaning_rooms.clone();

        self.executor
            .execute_command(SetModeWithRooms(Start, room_ids))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use async_trait::async_trait;
    use mockall::{mock, Sequence};
    use serde_json::json;

    mock! {
        pub Executor {}

        #[async_trait]
        impl CommandExecutorTrait for Executor {
            async fn execute_command(&mut self, command: Command) -> Result<serde_json::Value>;
        }
    }

    const PROPERTIES: &[&str] = &[
        "battary_life",
        "box_type",
        "run_state",
        "suction_grade",
        "is_mop",
        "water_grade",
    ];

    #[tokio::test]
    async fn test_status() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &GetProperties(PROPERTIES))
            .returning(|_| Ok(json!([100, 1, 0, 1, 1, 11])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.status().await.is_ok());
    }

    #[tokio::test]
    async fn test_set_fan_speed() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetFanSpeed(FanSpeed::Standard))
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.set_fan_speed(FanSpeed::Standard).await.is_ok());
    }

    #[tokio::test]
    async fn test_set_water_grade() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetWaterGrade(WaterGrade::Low))
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.set_water_grade(WaterGrade::Low).await.is_ok());
    }

    #[tokio::test]
    async fn test_set_clean_mode() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetCleanMode(CleanMode::Mop))
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.set_clean_mode(CleanMode::Mop).await.is_ok());
    }

    #[tokio::test]
    async fn test_go_home() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetCharge)
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.go_home().await.is_ok());
    }

    #[tokio::test]
    async fn test_start_vacuum_bin_vacuum_mode() {
        let mut seq = Sequence::new();
        let mut mock = MockExecutor::new();

        mock.expect_execute_command()
            .withf(|cmd| cmd == &GetProperties(PROPERTIES))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([100, 1, 0, 1, 0, 11])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Start, vec![10, 11, 12]))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.start(vec![10, 11, 12]).await.is_ok());
        assert_eq!(vacuum.last_cleaning_rooms, vec![10, 11, 12]);
    }

    #[tokio::test]
    async fn test_start_vacuum_and_water_bin_and_vacuum_and_mop_mode() {
        let mut seq = Sequence::new();
        let mut mock = MockExecutor::new();

        mock.expect_execute_command()
            .withf(|cmd| cmd == &GetProperties(PROPERTIES))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([100, 3, 0, 1, 1, 11])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Start, vec![10, 11, 12]))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.start(vec![10, 11, 12]).await.is_ok());
        assert_eq!(vacuum.last_cleaning_rooms, vec![10, 11, 12]);
    }

    #[tokio::test]
    async fn test_start_vacuum_bin_and_mop_mode() {
        let mut seq = Sequence::new();
        let mut mock = MockExecutor::new();

        mock.expect_execute_command()
            .withf(|cmd| cmd == &GetProperties(PROPERTIES))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([100, 1, 0, 1, 2, 11])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetCleanMode(CleanMode::Vacuum))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Start, vec![10, 11, 12]))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.start(vec![10, 11, 12]).await.is_ok());
        assert_eq!(vacuum.last_cleaning_rooms, vec![10, 11, 12]);
    }

    #[tokio::test]
    async fn test_start_vacuum_and_water_bin_and_mop_mode() {
        let mut seq = Sequence::new();
        let mut mock = MockExecutor::new();

        mock.expect_execute_command()
            .withf(|cmd| cmd == &GetProperties(PROPERTIES))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([100, 3, 0, 1, 0, 13])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetCleanMode(CleanMode::VacuumAndMop))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Start, vec![10, 11, 12]))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.start(vec![10, 11, 12]).await.is_ok());
        assert_eq!(vacuum.last_cleaning_rooms, vec![10, 11, 12]);
    }

    #[tokio::test]
    async fn test_start_vacuum_low_water_grade() {
        let mut seq = Sequence::new();
        let mut mock = MockExecutor::new();

        mock.expect_execute_command()
            .withf(|cmd| cmd == &GetProperties(PROPERTIES))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([100, 3, 0, 1, 0, 11])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetCleanMode(CleanMode::VacuumAndMop))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetWaterGrade(WaterGrade::High))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Start, vec![10, 11, 12]))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.start(vec![10, 11, 12]).await.is_ok());
        assert_eq!(vacuum.last_cleaning_rooms, vec![10, 11, 12]);
    }

    #[tokio::test]
    async fn test_start_vacuum_high_water_grade() {
        let mut seq = Sequence::new();
        let mut mock = MockExecutor::new();

        mock.expect_execute_command()
            .withf(|cmd| cmd == &GetProperties(PROPERTIES))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([100, 3, 0, 1, 0, 13])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetCleanMode(CleanMode::VacuumAndMop))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Start, vec![10, 11, 12]))
            .once()
            .in_sequence(&mut seq)
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.start(vec![10, 11, 12]).await.is_ok());
        assert_eq!(vacuum.last_cleaning_rooms, vec![10, 11, 12]);
    }

    #[tokio::test]
    async fn test_stop() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetMode(command::Mode::Stop))
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![],
        };

        assert!(vacuum.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_pause() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Pause, vec![3, 2, 1]))
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![3, 2, 1],
        };

        assert!(vacuum.pause().await.is_ok());
    }

    #[tokio::test]
    async fn test_resume() {
        let mut mock = MockExecutor::new();
        mock.expect_execute_command()
            .withf(|cmd| cmd == &SetModeWithRooms(command::Mode::Start, vec![3, 2, 1]))
            .returning(|_| Ok(json!([])));

        let mut vacuum = Vacuum {
            executor: Box::new(mock),
            last_cleaning_rooms: vec![3, 2, 1],
        };

        assert!(vacuum.resume().await.is_ok());
    }
}
