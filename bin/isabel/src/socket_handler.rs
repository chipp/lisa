use log::error;
use std::{fmt, sync::Arc};
use tokio::{
    io::{BufReader, BufWriter, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::{vacuum::Status, Result};
use elisheba::{
    decrypt, encrypt, read_bytes, write_bytes, Command, CommandResponse, PacketContent, SensorData,
    Token32, VacuumStatus,
};

type Reader = BufReader<ReadHalf<TcpStream>>;
type Writer = BufWriter<WriteHalf<TcpStream>>;

#[derive(Debug)]
struct NotConnected;

impl fmt::Display for NotConnected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Not connected to a socket")
    }
}

impl std::error::Error for NotConnected {}

#[derive(Clone)]
pub struct SocketHandler {
    reader: Arc<Mutex<Option<Reader>>>,
    writer: Arc<Mutex<Option<Writer>>>,
    token: Token32,
}

impl SocketHandler {
    pub fn new(token: Token32) -> SocketHandler {
        SocketHandler {
            reader: Arc::from(Mutex::from(None)),
            writer: Arc::from(Mutex::from(None)),
            token,
        }
    }

    pub async fn set_stream(&mut self, stream: TcpStream) {
        let (tcp_reader, tcp_writer) = tokio::io::split(stream);

        {
            let mut reader = self.reader.clone().lock_owned().await;
            *reader = Some(BufReader::new(tcp_reader))
        }

        {
            let mut writer = self.writer.clone().lock_owned().await;
            *writer = Some(BufWriter::new(tcp_writer))
        }
    }

    pub async fn read_commands<F>(&mut self, handler: impl Fn(Command) -> F) -> Result<()>
    where
        F: std::future::Future<Output = CommandResponse>,
    {
        loop {
            let bytes = if let Some(ref mut reader) = *self.reader.clone().lock_owned().await {
                read_bytes(reader).await?
            } else {
                return Err(Box::new(NotConnected));
            };

            if bytes.is_empty() {
                return Ok(());
            }

            let bytes = decrypt(bytes, self.token);

            let response = match bytes
                .and_then(|b| serde_json::from_slice::<Command>(&b).map_err(Into::into))
            {
                Ok(command) => handler(command).await,
                Err(err) => {
                    error!("unable to parse Command {}", err);
                    CommandResponse::Failure
                }
            };

            let bytes = serde_json::to_vec(&response.to_packet()).unwrap();

            if let Some(ref mut writer) = *self.writer.clone().lock_owned().await {
                write_bytes(writer, &bytes).await?;
            } else {
                return Err(Box::new(NotConnected));
            }
        }
    }

    pub async fn report_vacuum_status(&mut self, status: Status) -> Result<()> {
        let bytes = serde_json::to_vec(
            &VacuumStatus {
                battery: status.battery,
                is_enabled: status.state.is_enabled(),
                work_speed: status.fan_speed.to_string(),
            }
            .to_packet(),
        )
        .unwrap();

        let bytes = encrypt(bytes, self.token)?;

        if let Some(ref mut writer) = *self.writer.clone().lock_owned().await {
            write_bytes(writer, &bytes).await?;

            Ok(())
        } else {
            Err(Box::new(NotConnected))
        }
    }

    pub async fn report_sensor_data(&mut self, sensor_data: SensorData) -> Result<()> {
        let bytes = serde_json::to_vec(&sensor_data.to_packet()).unwrap();
        let bytes = encrypt(bytes, self.token)?;

        if let Some(ref mut writer) = *self.writer.clone().lock_owned().await {
            write_bytes(writer, &bytes).await?;

            Ok(())
        } else {
            Err(Box::new(NotConnected))
        }
    }
}
