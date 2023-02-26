use crate::OutgoingMessage;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlRequestMessage<'a> {
    pub message: MessageContent<'a>,
    pub sequence_id: u32,
}

impl<'q> SqlRequestMessage<'q> {
    pub fn new(query: &'q str, sequence_id: u32) -> SqlRequestMessage<'q> {
        SqlRequestMessage {
            message: MessageContent { query },
            sequence_id,
        }
    }
}

impl OutgoingMessage for SqlRequestMessage<'_> {
    fn code(&self) -> &'static str {
        "303"
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageContent<'a> {
    #[serde(rename = "query_local")]
    pub query: &'a str,
}
