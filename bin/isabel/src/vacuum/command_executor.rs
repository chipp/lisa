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

#[async_trait]
impl CommandExecutorTrait for CommandExecutor {
    async fn execute_command(&mut self, command: Command) -> Result<Value> {
        self.device.send(command.name(), command).await
    }
}
