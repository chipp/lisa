use std::{fmt, sync::Arc};

use log::error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::Result;
use elisheba::{Command, CommandResponse, Packet};

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

#[derive(Debug)]
struct CommandFailed;

impl fmt::Display for CommandFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command failed")
    }
}

impl std::error::Error for CommandFailed {}

#[derive(Clone)]
pub struct SocketHandler {
    reader: Arc<Mutex<Option<Reader>>>,
    writer: Arc<Mutex<Option<Writer>>>,
}

impl SocketHandler {
    pub fn new() -> SocketHandler {
        SocketHandler {
            reader: Arc::from(Mutex::from(None)),
            writer: Arc::from(Mutex::from(None)),
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

    pub async fn send_command(&mut self, command: &Command) -> Result<()> {
        let bytes = serde_json::to_vec(&command)?;

        if let Some(ref mut writer) = *self.writer.clone().lock_owned().await {
            writer.write_all(&bytes).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;

            Ok(())
        } else {
            Err(Box::new(NotConnected))
        }
    }

    pub fn handle_response(response: Option<CommandResponse>) -> Result<()> {
        match response {
            Some(CommandResponse::Ok) => Ok(()),
            Some(CommandResponse::Failure) => Err(Box::new(CommandFailed)),
            None => Err(Box::new(CommandFailed)),
        }
    }

    pub async fn read_packets<F>(&mut self, handler: impl Fn(Packet) -> F) -> Result<()>
    where
        F: std::future::Future<Output = ()>,
    {
        if let Some(ref mut reader) = *self.reader.clone().lock_owned().await {
            loop {
                let mut buffer = vec![];

                let size = reader.read_until(b'\n', &mut buffer).await?;

                if size == 0 {
                    return Ok(());
                }

                match serde_json::from_slice::<Packet>(&buffer) {
                    Ok(packet) => handler(packet).await,
                    Err(err) => {
                        error!("unable to parse Packet {}", err);
                        ()
                    }
                }
            }
        } else {
            Err(Box::new(NotConnected))
        }
    }
}
