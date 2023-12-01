use crate::device::Device;

use super::{Command, Result};

use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait CommandExecutorTrait {
    async fn execute_command(&mut self, command: Command) -> Result<Value>;
}

pub struct CommandExecutor {
    device: Device,
}

impl CommandExecutor {
    pub fn new(device: Device) -> Self {
        Self { device }
    }
}

#[cfg(feature = "stub")]
pub struct StubCommandExecutor;

#[cfg(feature = "stub")]
#[async_trait]
impl CommandExecutorTrait for StubCommandExecutor {
    async fn execute_command(&mut self, command: Command) -> Result<Value> {
        log::info!("stub command: {}", command.name());
        log::info!("stub command: {:?}", command);

        match command {
            Command::GetProperties(_) => Ok(json::json!([100, 1, 0, 1, 1, 11])),
            Command::SetFanSpeed(_) => Ok(json::json!([])),
            Command::SetWaterGrade(_) => Ok(json::json!([])),
            Command::SetCleanMode(_) => Ok(json::json!([])),
            Command::SetModeWithRooms(_, _) => Ok(json::json!([])),
            Command::SetMode(_) => Ok(json::json!([])),
            Command::SetCharge => Ok(json::json!([])),
        }
    }
}

#[async_trait]
impl CommandExecutorTrait for CommandExecutor {
    async fn execute_command(&mut self, command: Command) -> Result<Value> {
        self.device.send(command.name(), command).await
    }
}
