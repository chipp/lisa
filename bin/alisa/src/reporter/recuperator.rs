use crate::{DeviceId, Result};

use alice::{Mode, ModeFunction, StateCapability, StateDevice};
use serde_json::Value;
use topics::{ElizabethState, Room};

use serde::{Deserialize, Serialize};

pub fn prepare_recuperator_update(
    room: Option<Room>,
    state: ElizabethState,
    payload: Value,
) -> Result<StateDevice> {
    let state_capability;

    match state {
        ElizabethState::IsEnabled => {
            let value: bool = serde_json::from_value(payload)?;
            state_capability = StateCapability::on_off(value);
        }
        ElizabethState::FanSpeed => {
            let value: FanSpeed = serde_json::from_value(payload)?;
            state_capability = StateCapability::mode(ModeFunction::FanSpeed, value.to_mode());
        }
        ElizabethState::Temperature => todo!(),
        ElizabethState::CurrentTemperature => todo!(),
        ElizabethState::Mode => todo!(),
    }

    // TODO: throw an error
    let room = room.unwrap();
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
