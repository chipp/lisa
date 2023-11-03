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

impl Into<Mode> for FanSpeed {
    fn into(self) -> Mode {
        match self {
            FanSpeed::Silent => Mode::Quiet,
            FanSpeed::Standard => Mode::Normal,
            FanSpeed::Medium => Mode::Medium,
            FanSpeed::Turbo => Mode::Turbo,
        }
    }
}
