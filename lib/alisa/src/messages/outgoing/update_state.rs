use serde::Serialize;

use crate::OutMessage;
use crate::PortName;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStateMessage<'p, 'v> {
    pub message: UpdateStateMessageContent<'p, 'v>,
}

impl UpdateStateMessage<'_, '_> {
    pub fn new<'p, 'v>(
        force: bool,
        id: &'p str,
        r#type: PortName,
        value: &'v str,
    ) -> UpdateStateMessage<'p, 'v> {
        UpdateStateMessage {
            message: UpdateStateMessageContent {
                force,
                id,
                r#type,
                value,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateStateMessageContent<'p, 'v> {
    pub force: bool,
    pub id: &'p str,
    pub r#type: PortName,
    pub value: &'v str,
}

impl OutMessage for UpdateStateMessage<'_, '_> {
    fn code(&self) -> &'static str {
        "104"
    }
}
