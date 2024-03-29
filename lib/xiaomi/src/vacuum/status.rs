use std::fmt;
use std::str::FromStr;

use serde::de::{Deserializer, Error, SeqAccess, Visitor};
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug)]
pub struct Status {
    pub battery: u8,
    pub bin_type: BinType,
    pub state: State,
    pub fan_speed: FanSpeed,
    pub clean_mode: CleanMode,
    pub water_grade: WaterGrade,
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
            BinType::Vacuum => write!(f, "vacuum bin"),
            BinType::Water => write!(f, "water bin"),
            BinType::VacuumAndWater => write!(f, "vacuum and water bin"),
        }
    }
}

#[derive(Debug, Deserialize_repr, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum CleanMode {
    Vacuum = 0,
    VacuumAndMop = 1,
    Mop = 2,
    CleanZone = 3,
    CleanSpot = 4,
}

impl fmt::Display for CleanMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CleanMode::Vacuum => write!(f, "vacuum"),
            CleanMode::VacuumAndMop => write!(f, "vacuum and mop"),
            CleanMode::Mop => write!(f, "mop"),
            CleanMode::CleanZone => write!(f, "clean zone"),
            CleanMode::CleanSpot => write!(f, "clean spot"),
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

impl State {
    pub fn is_enabled(&self) -> bool {
        match self {
            State::Cleaning | State::VacuumingAndMopping => true,
            State::Unknown
            | State::IdleNotDocked
            | State::Idle
            | State::Idle2
            | State::Returning
            | State::Docked => false,
        }
    }

    pub fn is_paused(&self) -> bool {
        match self {
            State::IdleNotDocked | State::Idle | State::Idle2 => true,
            State::Unknown
            | State::Cleaning
            | State::Returning
            | State::Docked
            | State::VacuumingAndMopping => false,
        }
    }
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

impl FromStr for FanSpeed {
    type Err = serde::de::value::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "silent" => Ok(Self::Silent),
            "standard" => Ok(Self::Standard),
            "medium" => Ok(Self::Medium),
            "turbo" => Ok(Self::Turbo),
            _ => Err(serde::de::value::Error::custom(format!(
                "invalid FanSpeed string {}",
                value
            ))),
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

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq)]
#[repr(u8)]
pub enum WaterGrade {
    Low = 11,
    Medium = 12,
    High = 13,
}

impl fmt::Display for WaterGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaterGrade::Low => write!(f, "low"),
            WaterGrade::Medium => write!(f, "medium"),
            WaterGrade::High => write!(f, "high"),
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
        let clean_mode: CleanMode = seq.next_element()?.ok_or(Error::missing_field("is_mop"))?;
        let water_grade: WaterGrade = seq
            .next_element()?
            .ok_or(Error::missing_field("water_grade"))?;

        Ok(Status {
            battery,
            bin_type,
            state,
            fan_speed,
            clean_mode,
            water_grade,
        })
    }

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "expected an array of values")
    }
}

pub const FIELDS: &[&str] = &[
    "battary_life",
    "box_type",
    "run_state",
    "suction_grade",
    "is_mop",
    "water_grade",
];

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parsing() {
        let data = json!([99, 3, 6, 3, 2, 12]);
        let status: Status = serde_json::from_value(data).unwrap();

        assert_eq!(status.battery, 99);
        assert_eq!(status.bin_type, BinType::VacuumAndWater);
        assert_eq!(status.state, State::VacuumingAndMopping);
        assert_eq!(status.fan_speed, FanSpeed::Turbo);
        assert_eq!(status.clean_mode, CleanMode::Mop);
        assert_eq!(status.water_grade, WaterGrade::Medium);
    }
}
