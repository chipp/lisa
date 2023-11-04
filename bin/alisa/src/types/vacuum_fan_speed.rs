use alice::Mode;

use log::error;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FanSpeed {
    Silent,
    Standard,
    Medium,
    Turbo,
}

impl From<Mode> for FanSpeed {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Quiet => Self::Silent,
            Mode::Normal => Self::Standard,
            Mode::Medium => Self::Medium,
            Mode::Turbo => Self::Turbo,

            Mode::Low | Mode::High => {
                error!("unsupported Vacuum fan speed {}", value);
                Self::Silent
            }
        }
    }
}

impl From<FanSpeed> for Mode {
    fn from(val: FanSpeed) -> Self {
        match val {
            FanSpeed::Silent => Mode::Quiet,
            FanSpeed::Standard => Mode::Normal,
            FanSpeed::Medium => Mode::Medium,
            FanSpeed::Turbo => Mode::Turbo,
        }
    }
}
