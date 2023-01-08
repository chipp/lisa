use serde::Serialize;

use crate::OutMessage;
use crate::PortName;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStateMessage<'v> {
    pub message: UpdateStateMessageContent<'v>,
}

impl UpdateStateMessage<'_> {
    pub fn new<'v>(
        force: bool,
        id: &'v str,
        name: &'v PortName,
        value: &'v str,
    ) -> UpdateStateMessage<'v> {
        UpdateStateMessage {
            message: UpdateStateMessageContent {
                force,
                id,
                name,
                value,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateStateMessageContent<'v> {
    pub force: bool,
    pub id: &'v str,
    #[serde(rename = "type")]
    pub name: &'v PortName,
    pub value: &'v str,
}

impl OutMessage for UpdateStateMessage<'_> {
    fn code(&self) -> &'static str {
        "104"
    }
}
