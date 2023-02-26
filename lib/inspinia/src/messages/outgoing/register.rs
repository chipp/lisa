use crate::OutgoingMessage;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterMessage<'a> {
    device_type: &'a str,
    device_name: &'a str,
    push_token: &'a str,
}

impl RegisterMessage<'_> {
    pub fn new<'a>(
        device_type: &'a str,
        device_name: &'a str,
        push_token: &'a str,
    ) -> RegisterMessage<'a> {
        RegisterMessage {
            device_type,
            device_name,
            push_token,
        }
    }
}

impl OutgoingMessage for RegisterMessage<'_> {
    fn code(&self) -> &'static str {
        "101"
    }
}
