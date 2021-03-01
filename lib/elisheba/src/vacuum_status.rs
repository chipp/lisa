use serde::{Deserialize, Serialize};

use crate::{packet::PacketContent, Packet};

#[derive(Debug, Deserialize, Serialize)]
pub struct VacuumStatus {
    pub battery: u8,
    pub is_enabled: bool,
    pub work_speed: String,
}

impl PacketContent for VacuumStatus {
    fn to_packet(self) -> Packet {
        Packet::VacuumStatus(self)
    }
}
