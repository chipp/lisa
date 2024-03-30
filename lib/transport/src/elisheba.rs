use serde::{Deserialize, Serialize};

use crate::Room;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub is_enabled: bool,
    pub room: Room,
}
