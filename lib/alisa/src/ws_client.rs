use std::time::Instant;

use crate::{ReceivedMessage, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::debug;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub trait OutMessage {
    fn code(&self) -> &'static str;
}

pub struct WSClient {
    start: Instant,
    sequence: u32,
    target_id: String,
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WSClient {
    pub async fn connect<MI: AsRef<str>>(mobile_id: MI, target_id: String) -> Result<WSClient> {
        let uri = format!("wss://skyplatform.io:35601/mobileId={}", mobile_id.as_ref());
        let (web_socket, _) = connect_async(uri).await?;

        let (write, read) = web_socket.split();

        Ok(WSClient {
            start: Instant::now(),
            sequence: 0,
            target_id,
            write,
            read,
        })
    }

    pub async fn send_message<Msg>(&mut self, message: Msg) -> Result<()>
    where
        Msg: Serialize + OutMessage,
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

        self.write.send(Message::Text(text)).await?;

        Ok(())
    }

    pub async fn read_message(&mut self) -> Option<ReceivedMessage> {
        match self.read.next().await?.ok()? {
            Message::Text(text) => serde_json::from_str(&text).ok()?,
            Message::Ping(payload) => {
                self.write.send(Message::Pong(payload)).await.ok()?;
                None
            }
            message => {
                println!("unexpected message: {}", message);
                None
            }
        }
    }
}
