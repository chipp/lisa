use std::ffi::OsStr;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::{ReceivedMessage, Result};
use chrono::{DateTime, Local, TimeDelta};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{debug, info};
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

        let log = Arc::new(Mutex::new(Log::new(logs_path)));

        Ok(WsClient {
            start: Instant::now(),
            sequence: 0,
            target_id,
            write,
            read,
            log,
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
        let now = Local::now();

        log.add(now, message).await.unwrap();

        let duration = now - log.start;

        if log.size >= TEN_MIB || duration.num_days() >= 1 {
            let size = format_size(log.size);
            let duration = format_duration(duration);

            info!("saving log {}, started {}", size, duration);
            _ = log.archive().await;
            info!("saved log");
        }
    }
}

fn format_duration(duration: TimeDelta) -> String {
    HumanTime::from(duration).to_text_en(Accuracy::Rough, Tense::Past)
}

fn format_size(size: usize) -> String {
    let kb = size / 1024;
    let mb = kb / 1024;

    if mb > 0 {
        format!("{} MiB", mb)
    } else {
        format!("{} KiB", kb)
    }
}

const ONE_MIB: usize = 1024 * 1024;
const TEN_MIB: usize = 10 * ONE_MIB;

struct Log {
    start: DateTime<Local>,
    size: usize,
    path: PathBuf,
}

impl Log {
    fn new(root_path: PathBuf) -> Self {
        let start = Local::now();
        let mut path = root_path;
        path.push(format!(
            "inspinia-{}.txt",
            start.format("%Y-%m-%d-%H-%M-%S")
        ));

        Self {
            start,
            size: 0,
            path,
        }
    }

    async fn add(&mut self, ts: DateTime<Local>, message: &Message) -> Result<()> {
        let entry = format!("{}", LogEntry(message));
        let string = format!("{}: {}\n", ts.to_rfc2822(), entry);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        file.write_all(string.as_bytes()).await?;

        self.size += string.as_bytes().len();

        Ok(())
    }

    async fn archive(&mut self) -> Result<()> {
        let mut buf = Vec::with_capacity(TEN_MIB);

        {
            let mut cursor = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(&mut cursor);

            let filename = self
                .path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or("log.txt");

            zip.start_file(filename, Default::default())?;

            let mut buf = Vec::with_capacity(TEN_MIB + ONE_MIB);

            {
                let mut file = File::open(&self.path).await?;
                file.read_to_end(&mut buf).await?;
            }

            zip.write_all(&buf)?;

            zip.finish()?;
        }

        tokio::fs::remove_file(&self.path).await?;

        let mut path = self.path.clone();
        path.pop();
        path.push(format!(
            "inspinia-{}.zip",
            self.start.format("%Y-%m-%d-%H-%M-%S")
        ));

        let mut file = File::create(path).await.unwrap();
        file.write_all(&buf).await?;

        self.start = Local::now();
        self.size = 0;

        self.path.pop();
        self.path.push(format!(
            "inspinia-{}.txt",
            self.start.format("%Y-%m-%d-%H-%M-%S")
        ));

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
