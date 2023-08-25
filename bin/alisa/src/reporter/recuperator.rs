use crate::{Capability, DeviceId, Result, Room};

use alice::{Mode, ModeFunction, StateCapability, StateDevice};

use serde::{Deserialize, Serialize};

pub fn prepare_recuperator_update(
    room: Room,
    capability: Capability,
    payload: &[u8],
) -> Result<StateDevice> {
    let state_capability;

    match capability {
        Capability::IsEnabled => {
            let value: bool = serde_json::from_slice(payload)?;
            state_capability = StateCapability::on_off(value);
        }
        Capability::FanSpeed => {
            let value: FanSpeed = serde_json::from_slice(payload)?;
            state_capability = StateCapability::mode(ModeFunction::FanSpeed, value.to_mode());
        }
        Capability::Temperature => todo!(),
        Capability::CurrentTemperature => todo!(),
        Capability::Mode => todo!(),
    }

    let device_id = DeviceId::recuperator_at_room(room);
    Ok(StateDevice::new_with_capabilities(
        device_id.to_string(),
        vec![state_capability],
    ))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum FanSpeed {
    Low,
    Medium,
    High,
}

impl FanSpeed {
    fn to_mode(self) -> Mode {
        match self {
            FanSpeed::Low => Mode::Low,
            FanSpeed::Medium => Mode::Medium,
            FanSpeed::High => Mode::High,
        }
    }
}
