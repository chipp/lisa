use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum FanSpeed {
    Low,
    Medium,
    High,
}

#[derive(Debug)]
pub struct UnknownFanSpeed(String);

impl fmt::Display for UnknownFanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Unknown fan speed {}", self.0))
    }
}

impl Error for UnknownFanSpeed {}

impl TryFrom<&str> for FanSpeed {
    type Error = UnknownFanSpeed;

    fn try_from(value: &str) -> Result<Self, UnknownFanSpeed> {
        match value {
            "Low" => Ok(Self::Low),
            "Med" => Ok(Self::Medium),
            "High" => Ok(Self::High),
            _ => Err(UnknownFanSpeed(value.to_string())),
        }
    }
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FanSpeed::Low => f.write_str("Low"),
            FanSpeed::Medium => f.write_str("Med"),
            FanSpeed::High => f.write_str("High"),
        }
    }
}
