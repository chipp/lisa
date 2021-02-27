use log::error;
use std::{fmt, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::Result;
use elisheva::{Command, CommandResponse};

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

pub struct SocketHandler {
    reader: Option<Arc<Mutex<Reader>>>,
    writer: Option<Arc<Mutex<Writer>>>,
}

impl SocketHandler {
    pub fn new() -> SocketHandler {
        SocketHandler {
            reader: None,
            writer: None,
        }
    }

    pub fn set_stream(&mut self, stream: TcpStream) {
        let (reader, writer) = tokio::io::split(stream);

        self.reader = Some(Arc::from(Mutex::from(BufReader::new(reader))));
        self.writer = Some(Arc::from(Mutex::from(BufWriter::new(writer))));
    }

    pub async fn read_commands<F>(&mut self, handler: impl Fn(Command) -> F) -> Result<()>
    where
        F: std::future::Future<Output = CommandResponse>,
    {
        loop {
            let mut buffer = vec![];

            let size = if let Some(ref mut reader) = self.reader {
                let mut reader = reader.clone().lock_owned().await;
                reader.read_until(b'\n', &mut buffer).await?
            } else {
                return Err(Box::new(NotConnected));
            };

            if size == 0 {
                return Ok(());
            }

            let response = match serde_json::from_slice::<Command>(&buffer) {
                Ok(command) => handler(command).await,
                Err(err) => {
                    error!("unable to parse Command {}", err);
                    CommandResponse::Failure
                }
            };

            let bytes = serde_json::to_vec(&response).unwrap();

            if let Some(ref mut writer) = self.writer {
                let mut writer = writer.clone().lock_owned().await;
                Self::send_bytes(&mut writer, &bytes).await?;
            } else {
                return Err(Box::new(NotConnected));
            }
        }
    }

    async fn send_bytes(writer: &mut Writer, bytes: &[u8]) -> Result<()> {
        writer.write_all(&bytes).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        Ok(())
    }
}
