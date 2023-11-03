use topics::{Device, ElisaAction, ElizabethState, Room};

use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum Action {
    Elizabeth(ElizabethState),
    Elisa(ElisaAction),
}

impl From<ElisaAction> for Action {
    fn from(value: ElisaAction) -> Self {
        Self::Elisa(value)
    }
}

impl From<ElizabethState> for Action {
    fn from(value: ElizabethState) -> Self {
        Self::Elizabeth(value)
    }
}

pub struct ActionPayload {
    pub device: Device,
    pub room: Room,
    pub action: Action,
    pub value: Value,
}
