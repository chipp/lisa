use inspinia::{PortName, Room};
use topics::Device;

use crate::State;

#[derive(Debug)]
pub struct StatePayload {
    pub device: Device,
    pub room: Room,
    pub state: State,
    pub value: String,
}

pub fn port_name_to_state(port_name: PortName) -> State {
    match port_name {
        PortName::OnOff => State::IsEnabled,
        PortName::FanSpeed => State::FanSpeed,
        PortName::SetTemp => State::Temperature,
        PortName::RoomTemp => State::CurrentTemperature,
        PortName::Mode => State::Mode,
    }
}
