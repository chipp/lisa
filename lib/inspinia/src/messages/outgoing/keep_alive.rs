use crate::OutgoingMessage;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepAliveMessage {}

impl OutgoingMessage for KeepAliveMessage {
    fn code(&self) -> &'static str {
        "103"
    }
}
