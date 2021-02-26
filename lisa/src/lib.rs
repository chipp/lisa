mod device_id;
mod device_type;
mod room;
mod state;
mod update_state;

pub use device_id::DeviceId;
pub use device_type::DeviceType;
pub use room::Room;

pub use state::state_for_device;
pub use update_state::update_devices_state;

use elisheva::Command;

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

use tokio::{
    io::{AsyncWriteExt, BufWriter, WriteHalf},
    net::TcpStream,
};

pub struct Commander {
    stream: Option<BufWriter<WriteHalf<TcpStream>>>,
}

impl Commander {
    pub fn new() -> Commander {
        Commander { stream: None }
    }

    pub fn set_stream(&mut self, stream: BufWriter<WriteHalf<TcpStream>>) {
        self.stream = Some(stream)
    }

    pub async fn send_command(&mut self, command: Command) -> Result<()> {
        let bytes = serde_json::to_vec(&command)?;

        if let Some(ref mut stream) = self.stream {
            stream.write_all(&bytes).await?;
            stream.write_all(b"\n").await?;
            stream.flush().await?;
        }

        Ok(())
    }
}
