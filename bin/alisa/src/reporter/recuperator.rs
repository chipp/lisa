use crate::{DeviceId, Result};
use std::fmt;

use alice::{Mode, ModeFunction, StateCapability, StateDevice};
use transport::elizabeth::{Capability, State};

#[derive(Debug)]
pub enum Error {
    NotSupported(Capability),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotSupported(capability) => {
                write!(f, "capability {:?} is not supported", capability)
            }
        }
    }
}

impl std::error::Error for Error {}

pub fn prepare_recuperator_update(state: State) -> Result<StateDevice> {
    let state_capability = match state.capability {
        Capability::IsEnabled(value) => StateCapability::on_off(value),
        Capability::FanSpeed(fan_speed) => {
            StateCapability::mode(ModeFunction::FanSpeed, map_fan_speed(fan_speed))
        }
        Capability::Temperature(_) | Capability::CurrentTemperature(_) => {
            return Err(Error::NotSupported(state.capability).into())
        }
    };

    let device_id = DeviceId::recuperator_at_room(state.room);

    Ok(StateDevice::new_with_capabilities(
        device_id.to_string(),
        vec![state_capability],
    ))
}

fn map_fan_speed(speed: transport::elizabeth::FanSpeed) -> Mode {
    match speed {
        transport::elizabeth::FanSpeed::Low => Mode::Low,
        transport::elizabeth::FanSpeed::Medium => Mode::Medium,
        transport::elizabeth::FanSpeed::High => Mode::High,
    }
}
