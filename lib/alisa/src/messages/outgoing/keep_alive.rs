use crate::OutgoingMessage;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepAliveMessage {}

impl KeepAliveMessage {
    pub fn new() -> KeepAliveMessage {
        KeepAliveMessage {}
    }
}

impl OutgoingMessage for KeepAliveMessage {
    fn code(&self) -> &'static str {
        "103"
    }
}
