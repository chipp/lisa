use serde_json::{json, Map, Value};
use transport::elzhbieta::{ActionType, Mode, State};
use transport::Room;

use crate::error::{Error, Result};

const DP_SWITCH: &str = "1";
const DP_TARGET_TEMPERATURE: &str = "2";
const DP_CURRENT_TEMPERATURE: &str = "3";
const DP_MODE: &str = "4";
const DP_HUMIDITY: &str = "18";

pub fn state_from_dps(room: Room, dps: &Map<String, Value>) -> Result<State> {
    Ok(State {
        room,
        is_enabled: bool_dps(dps, DP_SWITCH, "switch")?,
        mode: mode_from_tuya(string_dps(dps, DP_MODE, "mode")?)?,
        target_temperature: int_dps(dps, DP_TARGET_TEMPERATURE, "temp_set")? as f32 / 10.0,
        current_temperature: int_dps(dps, DP_CURRENT_TEMPERATURE, "temp_current")? as f32,
        humidity: int_dps(dps, DP_HUMIDITY, "humidity_current").unwrap_or_default() as u8,
    })
}

pub fn dps_from_action(action: ActionType) -> Map<String, Value> {
    let mut dps = Map::new();

    match action {
        ActionType::SetIsEnabled(value) => {
            dps.insert(DP_SWITCH.to_string(), json!(value));
        }
        ActionType::SetMode(mode) => {
            dps.insert(DP_MODE.to_string(), json!(mode_to_tuya(mode)));
        }
        ActionType::SetTargetTemperature(value) => {
            dps.insert(
                DP_TARGET_TEMPERATURE.to_string(),
                json!((value * 10.0).round() as i32),
            );
        }
    }

    dps
}

fn bool_dps(dps: &Map<String, Value>, id: &'static str, name: &'static str) -> Result<bool> {
    dps.get(id)
        .and_then(Value::as_bool)
        .ok_or(Error::MissingDps(name))
}

fn int_dps(dps: &Map<String, Value>, id: &'static str, name: &'static str) -> Result<i64> {
    dps.get(id)
        .and_then(Value::as_i64)
        .ok_or(Error::MissingDps(name))
}

fn string_dps<'a>(
    dps: &'a Map<String, Value>,
    id: &'static str,
    name: &'static str,
) -> Result<&'a str> {
    dps.get(id)
        .and_then(Value::as_str)
        .ok_or(Error::MissingDps(name))
}

fn mode_from_tuya(value: &str) -> Result<Mode> {
    match value {
        "cold" => Ok(Mode::Cool),
        "hot" => Ok(Mode::Heat),
        "wet" => Ok(Mode::Dry),
        "wind" => Ok(Mode::FanOnly),
        "auto" => Ok(Mode::Auto),
        other => Err(Error::UnknownMode(other.to_string())),
    }
}

fn mode_to_tuya(mode: Mode) -> &'static str {
    match mode {
        Mode::Cool => "cold",
        Mode::Heat => "hot",
        Mode::Dry => "wet",
        Mode::FanOnly => "wind",
        Mode::Auto => "auto",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_state_from_dps() {
        let dps = json!({
            "1": true,
            "2": 220,
            "3": 27,
            "4": "cold",
            "18": 45
        });
        let dps = dps.as_object().unwrap();

        assert_eq!(
            state_from_dps(Room::Bedroom, dps).unwrap(),
            State {
                room: Room::Bedroom,
                is_enabled: true,
                mode: Mode::Cool,
                target_temperature: 22.0,
                current_temperature: 27.0,
                humidity: 45,
            }
        );
    }

    #[test]
    fn test_dps_from_action() {
        assert_eq!(
            Value::Object(dps_from_action(ActionType::SetIsEnabled(true))),
            json!({"1": true})
        );
        assert_eq!(
            Value::Object(dps_from_action(ActionType::SetMode(Mode::Heat))),
            json!({"4": "hot"})
        );
        assert_eq!(
            Value::Object(dps_from_action(ActionType::SetTargetTemperature(22.5))),
            json!({"2": 225})
        );
    }
}
