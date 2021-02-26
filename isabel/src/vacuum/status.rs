use std::fmt;

use serde::de::{Deserializer, Error, SeqAccess, Visitor};
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

pub struct Status {
    pub battery: u8,
    pub bin_type: BinType,
    pub state: State,
    pub fan_speed: FanSpeed,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "battery {}", self.battery)?;
        writeln!(f, "bin_type {}", self.bin_type)?;
        writeln!(f, "state {}", self.state)?;
        writeln!(f, "fan_speed {}", self.fan_speed)?;

        Ok(())
    }
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum BinType {
    NoBin = 0,
    Vacuum = 1,
    Water = 2,
    VacuumAndWater = 3,
}

impl fmt::Display for BinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinType::NoBin => write!(f, "no bin"),
            BinType::Vacuum => write!(f, "vacuum"),
            BinType::Water => write!(f, "water"),
            BinType::VacuumAndWater => write!(f, "vacuum and water"),
        }
    }
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(i8)]
pub enum State {
    Unknown = -1,
    IdleNotDocked = 0,
    Idle = 1,
    Idle2 = 2,
    Cleaning = 3,
    Returning = 4,
    Docked = 5,
    VacuumingAndMopping = 6,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Unknown => write!(f, "<unknown>"),
            State::IdleNotDocked => write!(f, "idle and not docked"),
            State::Idle | State::Idle2 => write!(f, "idle"),
            State::Cleaning => write!(f, "cleaning"),
            State::Returning => write!(f, "returning to dock"),
            State::Docked => write!(f, "docked"),
            State::VacuumingAndMopping => write!(f, "cleaning and mopping"),
        }
    }
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq)]
#[repr(u8)]
pub enum FanSpeed {
    Silent = 0,
    Standard = 1,
    Medium = 2,
    Turbo = 3,
}

impl From<String> for FanSpeed {
    fn from(string: String) -> Self {
        match string.as_str() {
            "quiet" => Self::Silent,
            "medium" => Self::Standard,
            "high" => Self::Medium,
            "turbo" => Self::Turbo,
            _ => panic!("invalid FanSpeed string {}", string),
        }
    }
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FanSpeed::Silent => write!(f, "silent"),
            FanSpeed::Standard => write!(f, "standard"),
            FanSpeed::Medium => write!(f, "medium"),
            FanSpeed::Turbo => write!(f, "turbo"),
        }
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StatusVisitor)
    }
}

struct StatusVisitor;

impl<'de> Visitor<'de> for StatusVisitor {
    type Value = Status;

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let battery: u8 = seq.next_element()?.ok_or(Error::missing_field("battery"))?;
        let bin_type: BinType = seq
            .next_element()?
            .ok_or(Error::missing_field("bin_type"))?;
        let state: State = seq.next_element()?.ok_or(Error::missing_field("state"))?;
        let fan_speed: FanSpeed = seq
            .next_element()?
            .ok_or(Error::missing_field("fan_speed"))?;

        Ok(Status {
            battery,
            bin_type,
            state,
            fan_speed,
        })
    }

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "expected an array of values")
    }
}

pub const FIELDS: &[&str] = &["battary_life", "box_type", "run_state", "suction_grade"];

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parsing() {
        let data = json!([99, 3, 6, 3]);
        let status: Status = serde_json::from_value(data).unwrap();

        assert_eq!(status.battery, 99);
        assert_eq!(status.bin_type, BinType::VacuumAndWater);
        assert_eq!(status.state, State::VacuumingAndMopping);
        assert_eq!(status.fan_speed, FanSpeed::Turbo);
    }
}
