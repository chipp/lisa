use serde::{Deserialize, Serialize};

use crate::{CommandResponse, VacuumStatus};

pub trait PacketContent {
    fn to_packet(self) -> Packet;
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Packet {
    CommandResponse(CommandResponse),
    VacuumStatus(VacuumStatus),
}
