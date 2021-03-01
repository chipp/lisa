use serde::{Deserialize, Serialize};

use crate::CommandResponse;

#[derive(Debug, Deserialize, Serialize)]
pub enum Packet {
    CommandResponse(CommandResponse),
    VacuumBatteryPercentage(u8),
}
