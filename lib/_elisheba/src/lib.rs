mod bytes;
mod command;
mod crypto;
mod packet;
mod sensor_data;
mod token;
mod vacuum_status;

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

pub use bytes::{hexdump, read_bytes, write_bytes};
pub use command::{Command, CommandResponse};
pub use crypto::{decrypt, encrypt};
pub use packet::{Packet, PacketContent};
pub use sensor_data::{SensorData, SensorRoom};
pub use token::{parse_token, Token};
pub use vacuum_status::VacuumStatus;
