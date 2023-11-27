use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::{ReceivedMessage, Result};
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
        WsError::WebSocketError(value)
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
}

impl WsClient {
    pub async fn connect<MI: AsRef<str>>(mobile_id: MI, target_id: String) -> Result<WsClient> {
        let uri = format!("wss://skyplatform.io:35601/mobileId={}", mobile_id.as_ref());
        let (web_socket, _) = connect_async(uri).await?;

        let (write, read) = web_socket.split();

        let write = Arc::from(Mutex::new(write));
        let read = Arc::from(Mutex::new(read));

        Ok(WsClient {
            start: Instant::now(),
            sequence: 0,
            target_id,
            write,
            read,
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
            Ok(Message::Text(text)) => {
                let message: ReceivedMessage = serde_json::from_str(&text)?;
                Ok(message)
            }
            Ok(Message::Ping(payload)) => {
                let mut write = self.write.lock().await;
                write.send(Message::Pong(payload)).await?;

                Err(WsError::Pong)
            }
            Ok(message) => Err(WsError::UnexpectedMessage(message)),
            Err(error) => Err(error)?,
        }
    }
}
