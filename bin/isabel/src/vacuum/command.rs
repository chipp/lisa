use super::CleanMode;
use super::FanSpeed;
use super::WaterGrade;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde_repr::Serialize_repr;

#[derive(PartialEq)]
pub enum Command {
    GetProperties(&'static [&'static str]),
    SetFanSpeed(FanSpeed),
    SetWaterGrade(WaterGrade),
    SetCleanMode(CleanMode),
    SetModeWithRooms(Mode, Vec<u8>),
    SetMode(Mode),
    SetCharge,
}

#[derive(Debug, Serialize_repr, PartialEq)]
#[repr(u8)]
pub enum Mode {
    Start = 1,
    Stop = 0,
    Pause = 2,
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Command::GetProperties(_) => "get_prop",
            Command::SetFanSpeed(_) => "set_suction",
            Command::SetWaterGrade(_) => "set_suction",
            Command::SetCleanMode(_) => "set_mop",
            Command::SetModeWithRooms(_, _) => "set_mode_withroom",
            Command::SetMode(_) => "set_mode",
            Command::SetCharge => "set_charge",
        }
    }
}

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Command::GetProperties(properties) => {
                let mut seq = serializer.serialize_seq(Some(properties.len()))?;

                for property in properties.iter() {
                    seq.serialize_element(property)?;
                }

                seq.end()
            }
            Command::SetFanSpeed(fan_speed) => {
                let mut seq = serializer.serialize_seq(Some(1))?;

                seq.serialize_element(&fan_speed)?;

                seq.end()
            }
            Command::SetWaterGrade(water_grade) => {
                let mut seq = serializer.serialize_seq(Some(1))?;

                seq.serialize_element(&water_grade)?;

                seq.end()
            }
            Command::SetCleanMode(mode) => {
                let mut seq = serializer.serialize_seq(Some(1))?;

                seq.serialize_element(&mode)?;

                seq.end()
            }
            Command::SetModeWithRooms(mode, rooms) => {
                let mut seq = serializer.serialize_seq(Some(6))?;

                seq.serialize_element(&0)?; // vacuum along edges – no
                seq.serialize_element(&mode)?;

                seq.serialize_element(&rooms.len())?;

                for room in rooms.iter() {
                    seq.serialize_element(&room)?;
                }

                seq.end()
            }
            Command::SetMode(mode) => {
                let mut seq = serializer.serialize_seq(Some(2))?;

                seq.serialize_element(&0)?; // vacuum along edges – no
                seq.serialize_element(&mode)?;

                seq.end()
            }
            Command::SetCharge => {
                let mut seq = serializer.serialize_seq(Some(1))?;

                seq.serialize_element(&1)?;

                seq.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};

    use super::*;

    #[test]
    fn test_get_properties() {
        let command = Command::GetProperties(&["a", "b", "c"]);

        assert_eq!(command.name(), "get_prop");

        let serialized = to_value(command).unwrap();
        assert_eq!(serialized, json!(["a", "b", "c"]));
    }

    #[test]
    fn test_set_fan_speed() {
        let command = Command::SetFanSpeed(super::FanSpeed::Medium);

        assert_eq!(command.name(), "set_suction");

        let serialized = to_value(command).unwrap();
        assert_eq!(serialized, json!([2]));
    }

    #[test]
    fn test_set_water_grade() {
        let command = Command::SetWaterGrade(super::WaterGrade::High);

        assert_eq!(command.name(), "set_suction");

        let serialized = to_value(command).unwrap();
        assert_eq!(serialized, json!([13]));
    }

    #[test]
    fn test_set_clean_mode() {
        let command = Command::SetCleanMode(CleanMode::VacuumAndMop);

        assert_eq!(command.name(), "set_mop");

        let serialized = to_value(command).unwrap();
        assert_eq!(serialized, json!([1]));
    }

    #[test]
    fn test_modes() {
        assert_eq!(to_value(Mode::Start).unwrap(), json!(1));
        assert_eq!(to_value(Mode::Stop).unwrap(), json!(0));
        assert_eq!(to_value(Mode::Pause).unwrap(), json!(2));
    }

    #[test]
    fn test_set_mode_with_rooms() {
        let command = Command::SetModeWithRooms(Mode::Pause, vec![1, 11, 21]);

        assert_eq!(command.name(), "set_mode_withroom");

        let serialized = to_value(command).unwrap();
        assert_eq!(serialized, json!([0, 2, 3, 1, 11, 21]));
    }

    #[test]
    fn test_set_mode() {
        let command = Command::SetMode(Mode::Start);

        assert_eq!(command.name(), "set_mode");

        let serialized = to_value(command).unwrap();
        assert_eq!(serialized, json!([0, 1]));
    }

    #[test]
    fn test_set_charge() {
        let command = Command::SetCharge;

        assert_eq!(command.name(), "set_charge");

        let serialized = to_value(command).unwrap();
        assert_eq!(serialized, json!([1]));
    }
}
