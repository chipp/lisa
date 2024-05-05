use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::{ReceivedMessage, Result};
use chrono::{DateTime, Local};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::debug;
use serde::Serialize;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub enum WsError {
    StreamClosed,
    CannotParse(serde_json::Error),
    WebSocketError(tokio_tungstenite::tungstenite::error::Error),
    UnexpectedMessage(Message),
    Pong,
}

impl fmt::Display for WsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WsError::StreamClosed => write!(f, "stream closed"),
            WsError::CannotParse(error) => write!(f, "cannot parse: {}", error),
            WsError::WebSocketError(error) => write!(f, "websocket error: {}", error),
            WsError::UnexpectedMessage(message) => write!(f, "unexpected message: {:?}", message),
            WsError::Pong => write!(f, "pong"),
        }
    }
}

impl std::error::Error for WsError {}

impl From<serde_json::Error> for WsError {
    fn from(value: serde_json::Error) -> Self {
        WsError::CannotParse(value)
    }
}

impl From<tokio_tungstenite::tungstenite::error::Error> for WsError {
    fn from(value: tokio_tungstenite::tungstenite::error::Error) -> Self {
        if let tokio_tungstenite::tungstenite::error::Error::AlreadyClosed = value {
            WsError::StreamClosed
        } else {
            WsError::WebSocketError(value)
        }
    }
}

pub trait OutgoingMessage {
    fn code(&self) -> &'static str;
}

type Writer = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type Reader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Clone)]
pub struct WsClient {
    start: Instant,
    sequence: u32,
    target_id: String,
    write: Arc<Mutex<Writer>>,
    read: Arc<Mutex<Reader>>,
    log: Arc<Mutex<Log>>,
    logs_path: PathBuf,
}

impl WsClient {
    pub async fn connect<MI: AsRef<str>>(
        mobile_id: MI,
        target_id: String,
        logs_path: PathBuf,
    ) -> Result<WsClient> {
        let uri = format!("wss://skyplatform.io:35601/mobileId={}", mobile_id.as_ref());
        let (web_socket, _) = connect_async(uri).await?;

        let (write, read) = web_socket.split();

        let write = Arc::from(Mutex::new(write));
        let read = Arc::from(Mutex::new(read));

        let log = Arc::new(Mutex::new(Log::new()));

        Ok(WsClient {
            start: Instant::now(),
            sequence: 0,
            target_id,
            write,
            read,
            log,
            logs_path,
        })
    }

    pub async fn send_message<Msg>(&mut self, message: Msg) -> Result<()>
    where
        Msg: Serialize + OutgoingMessage,
    {
        let code = message.code().to_string();
        let mut json = serde_json::to_value(message)?;

        if let Some(object) = json.as_object_mut() {
            object.insert("code".to_string(), serde_json::Value::String(code));

            object.insert(
                "type".to_string(),
                serde_json::Value::String("com.astrum.websocket.JSONRequest".to_string()),
            );

            object.insert(
                "sequence".to_string(),
                serde_json::Value::Number(self.sequence.into()),
            );
            self.sequence += 1;

            object.insert(
                "time".to_string(),
                serde_json::Value::Number(self.start.elapsed().as_secs().into()),
            );

            object.insert(
                "targetId".to_string(),
                serde_json::Value::String(self.target_id.to_string()),
            );
        }

        let text = serde_json::to_string(&json)?;
        debug!("sent {}", text);

        let mut write = self.write.lock().await;
        write.send(Message::Text(text)).await?;

        Ok(())
    }

    pub async fn read_message(&mut self) -> std::result::Result<ReceivedMessage, WsError> {
        let mut read = self.read.lock().await;

        match read.next().await.ok_or(WsError::StreamClosed)? {
            Ok(message) => {
                self.log_socket_message(&message).await;

                match message {
                    Message::Text(text) => {
                        let message: ReceivedMessage = serde_json::from_str(&text)?;
                        Ok(message)
                    }
                    Message::Ping(payload) => {
                        let mut write = self.write.lock().await;
                        write.send(Message::Pong(payload)).await?;

                        Err(WsError::Pong)
                    }
                    message => Err(WsError::UnexpectedMessage(message)),
                }
            }
            Err(error) => Err(error)?,
        }
    }

    async fn log_socket_message(&self, message: &Message) {
        let mut log = self.log.lock().await;
        log.add(message);

        if log.entries.len() > 500 {
            _ = log.save(&self.logs_path).await;
        }
    }
}

struct Log {
    start: DateTime<chrono::Local>,
    entries: Vec<(DateTime<Local>, String)>,
}

impl Log {
    fn new() -> Self {
        Self {
            start: chrono::Local::now(),
            entries: Vec::new(),
        }
    }

    fn add(&mut self, message: &Message) {
        let ts = chrono::Local::now();
        let entry = format!("{}", LogEntry(message));
        self.entries.push((ts, entry));
    }

    async fn save(&mut self, path: &Path) -> Result<()> {
        let mut buf = Vec::with_capacity(1024 * 1024);

        {
            let mut cursor = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(&mut cursor);

            zip.start_file("log.txt", Default::default())?;

            for (ts, entry) in &self.entries {
                zip.write_all(ts.to_rfc2822().as_bytes())?;
                zip.write_all(b": ")?;
                zip.write_all(entry.as_bytes())?;
                zip.write_all(b"\n")?;
            }

            zip.finish()?;
        }

        let mut path = path.to_path_buf();
        path.push(format!(
            "inspinia-{}.zip",
            self.start.format("%Y-%m-%d-%H-%M-%S")
        ));

        let mut file = File::create(path).await.unwrap();
        file.write_all(&buf).await?;

        self.start = chrono::Local::now();
        self.entries.clear();

        Ok(())
    }
}

struct LogEntry<'m>(&'m Message);

impl std::fmt::Display for LogEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Message::Text(payload) => write!(f, "text: {payload}"),
            Message::Binary(payload) => write!(f, "binary: {}", bytes_as_hex_string(payload)),
            Message::Ping(payload) => write!(f, "ping: {}", bytes_as_hex_string(payload)),
            Message::Pong(payload) => write!(f, "pong: {}", bytes_as_hex_string(payload)),
            Message::Close(Some(payload)) => write!(f, "close: {}", payload.reason),
            Message::Close(None) => write!(f, "close: N/A"),
            Message::Frame(payload) => {
                write!(f, "frame: {}", bytes_as_hex_string(payload.payload()))
            }
        }
    }
}

fn bytes_as_hex_string(bytes: &[u8]) -> String {
    let mut result = String::new();

    for byte in bytes {
        result.push_str(&format!("{:02x}", byte));
    }

    result
}
