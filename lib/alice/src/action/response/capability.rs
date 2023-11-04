use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::{ModeFunction, RangeFunction, ToggleFunction};

#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    Ok,
    Err(Error),
}

impl ActionResult {
    pub fn ok() -> ActionResult {
        ActionResult::Ok
    }

    pub fn error(code: ErrorCode, message: String) -> ActionResult {
        ActionResult::Err(Error::new(code, message))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    code: ErrorCode,
    message: String,
}

impl Error {
    pub fn new(code: ErrorCode, message: String) -> Error {
        Error { code, message }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidAction,
    InvalidValue,
    DeviceUnreachable,
    DeviceBusy,
}

#[derive(Debug, PartialEq)]
pub enum Capability {
    OnOff {
        result: ActionResult,
    },
    Mode {
        function: ModeFunction,
        result: ActionResult,
    },
    Toggle {
        function: ToggleFunction,
        result: ActionResult,
    },
    Range {
        function: RangeFunction,
        result: ActionResult,
    },
}

impl Capability {
    pub fn on_off(result: ActionResult) -> Capability {
        Capability::OnOff { result }
    }

    pub fn mode(function: ModeFunction, result: ActionResult) -> Capability {
        Capability::Mode { function, result }
    }

    pub fn toggle(function: ToggleFunction, result: ActionResult) -> Capability {
        Capability::Toggle { function, result }
    }

    pub fn range(function: RangeFunction, result: ActionResult) -> Capability {
        Capability::Range { function, result }
    }

    pub fn result_mut(&mut self) -> &mut ActionResult {
        match self {
            Capability::OnOff { result }
            | Capability::Mode { result, .. }
            | Capability::Toggle { result, .. }
            | Capability::Range { result, .. } => result,
        }
    }
}

impl serde::ser::Serialize for ActionResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut result = serializer.serialize_struct("ActionResult", 3)?;

        match self {
            ActionResult::Ok => {
                result.serialize_field("status", "DONE")?;
            }
            ActionResult::Err(error) => {
                result.serialize_field("status", "ERROR")?;
                result.serialize_field("error_code", &error.code)?;
                result.serialize_field("error_message", &error.message)?;
            }
        }

        result.end()
    }
}

#[derive(Serialize)]
struct State<S, U> {
    instance: S,
    action_result: U,
}

impl serde::ser::Serialize for Capability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut property = serializer.serialize_struct("Capability", 4)?;

        match self {
            Capability::OnOff { result } => {
                property.serialize_field("type", "devices.capabilities.on_off")?;
                property.serialize_field(
                    "state",
                    &State {
                        instance: "on",
                        action_result: result,
                    },
                )?;
            }
            Capability::Mode { function, result } => {
                property.serialize_field("type", "devices.capabilities.mode")?;
                property.serialize_field(
                    "state",
                    &State {
                        instance: function,
                        action_result: result,
                    },
                )?;
            }
            Capability::Toggle { function, result } => {
                property.serialize_field("type", "devices.capabilities.toggle")?;
                property.serialize_field(
                    "state",
                    &State {
                        instance: function,
                        action_result: result,
                    },
                )?;
            }
            Capability::Range { function, result } => {
                property.serialize_field("type", "devices.capabilities.range")?;
                property.serialize_field(
                    "state",
                    &State {
                        instance: function,
                        action_result: result,
                    },
                )?;
            }
        }

        property.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn test_action_result() {
        assert_eq!(
            to_value(&ActionResult::Ok).unwrap(),
            json!({"status": "DONE"})
        );

        let error = Error {
            code: ErrorCode::InvalidAction,
            message: "human readable test message".to_owned(),
        };

        assert_eq!(
            to_value(ActionResult::Err(error)).unwrap(),
            json!({
                "status": "ERROR",
                "error_code": "INVALID_ACTION",
                "error_message": "human readable test message"
            })
        );
    }

    #[test]
    fn test_capabilities() {
        assert_eq!(
            to_value(&Capability::OnOff {
                result: ActionResult::Ok
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.on_off",
                "state": {
                    "instance": "on",
                    "action_result": {"status": "DONE"}
                }
            })
        );

        assert_eq!(
            to_value(&Capability::Mode {
                function: ModeFunction::WorkSpeed,
                result: ActionResult::Ok
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.mode",
                "state": {
                    "instance": "work_speed",
                    "action_result": {"status": "DONE"}
                }
            })
        );

        assert_eq!(
            to_value(&Capability::Toggle {
                function: ToggleFunction::Pause,
                result: ActionResult::Ok
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.toggle",
                "state": {
                    "instance": "pause",
                    "action_result": {"status": "DONE"}
                }
            })
        );

        assert_eq!(
            to_value(&Capability::Range {
                function: RangeFunction::Temperature,
                result: ActionResult::Ok
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.range",
                "state": {
                    "instance": "temperature",
                    "action_result": {"status": "DONE"}
                }
            })
        );
    }
}
