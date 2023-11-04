use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FanSpeed {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq)]
pub struct UnknownFanSpeed(String);

impl fmt::Display for UnknownFanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Unknown fan speed {}", self.0))
    }
}

impl Error for UnknownFanSpeed {}

impl FromStr for FanSpeed {
    type Err = UnknownFanSpeed;

    fn from_str(value: &str) -> Result<Self, UnknownFanSpeed> {
        match value {
            "Low" | "low" => Ok(Self::Low),
            "Med" | "medium" => Ok(Self::Medium),
            "High" | "high" => Ok(Self::High),
            _ => Err(UnknownFanSpeed(value.to_string())),
        }
    }
}

// TODO: prepare a separate method for Inspinia
impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("Low"),
            Self::Medium => f.write_str("Med"),
            Self::High => f.write_str("High"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_speed_parsing() {
        assert_eq!(FanSpeed::from_str("Low"), Ok(FanSpeed::Low));
        assert_eq!(FanSpeed::from_str("low"), Ok(FanSpeed::Low));
        assert_eq!(FanSpeed::from_str("Med"), Ok(FanSpeed::Medium));
        assert_eq!(FanSpeed::from_str("medium"), Ok(FanSpeed::Medium));
        assert_eq!(FanSpeed::from_str("High"), Ok(FanSpeed::High));
        assert_eq!(FanSpeed::from_str("high"), Ok(FanSpeed::High));
        assert_eq!(
            FanSpeed::from_str("unknown"),
            Err(UnknownFanSpeed("unknown".to_string()))
        );
    }
}
