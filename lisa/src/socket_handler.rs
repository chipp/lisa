use std::fmt;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf},
    net::TcpStream,
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

#[derive(Debug)]
struct CommandFailed;

impl fmt::Display for CommandFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command failed")
    }
}

impl std::error::Error for CommandFailed {}

pub struct SocketHandler {
    reader: Option<Reader>,
    writer: Option<Writer>,
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

        self.reader = Some(BufReader::new(reader));
        self.writer = Some(BufWriter::new(writer));
    }

    pub async fn send_command(&mut self, command: Command) -> Result<()> {
        let bytes = serde_json::to_vec(&command)?;

        match (&mut self.reader, &mut self.writer) {
            (Some(reader), Some(writer)) => {
                writer.write_all(&bytes).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;

                let mut buffer = vec![];
                let size = reader.read_until(b'\n', &mut buffer).await?;

                if size == 0 {
                    return Err(Box::new(NotConnected));
                }

                match serde_json::from_slice(&buffer)? {
                    CommandResponse::Ok => Ok(()),
                    CommandResponse::Failure => Err(Box::new(CommandFailed)),
                }
            }
            _ => return Err(Box::new(NotConnected)),
        }
    }
}
