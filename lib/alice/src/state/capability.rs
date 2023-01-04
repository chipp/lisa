use std::str::FromStr;

use serde::de;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use crate::{Mode, ModeFunction, RangeFunction, ToggleFunction};

#[derive(Debug, Clone, PartialEq)]
pub enum Capability {
    OnOff {
        value: bool,
    },
    Mode {
        function: ModeFunction,
        mode: Mode,
    },
    Toggle {
        function: ToggleFunction,
        value: bool,
    },
    Range {
        function: RangeFunction,
        value: f32,
        relative: bool,
    },
}

impl Capability {
    pub fn on_off(value: bool) -> Capability {
        Capability::OnOff { value }
    }

    pub fn mode(function: ModeFunction, mode: Mode) -> Capability {
        Capability::Mode { function, mode }
    }

    pub fn toggle(function: ToggleFunction, value: bool) -> Capability {
        Capability::Toggle { function, value }
    }

    pub fn range(function: RangeFunction, value: f32) -> Capability {
        Capability::Range {
            function,
            value,
            relative: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct State<S, U> {
    instance: S,
    value: U,

    #[serde(skip_serializing)]
    #[serde(default)]
    relative: bool,
}

impl<S, U> State<S, U> {
    fn new(instance: S, value: U) -> State<S, U> {
        State {
            instance,
            value,
            relative: false,
        }
    }
}

impl serde::ser::Serialize for Capability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut property = serializer.serialize_struct("Capability", 4)?;

        match self {
            Capability::OnOff { value } => {
                property.serialize_field("type", "devices.capabilities.on_off")?;
                property.serialize_field("state", &State::new("on", value))?;
            }
            Capability::Mode { function, mode } => {
                property.serialize_field("type", "devices.capabilities.mode")?;
                property.serialize_field("state", &State::new(function, mode))?;
            }
            Capability::Toggle { function, value } => {
                property.serialize_field("type", "devices.capabilities.toggle")?;
                property.serialize_field("state", &State::new(function, value))?;
            }
            Capability::Range {
                function,
                value,
                relative: _,
            } => {
                property.serialize_field("type", "devices.capabilities.range")?;
                property.serialize_field("state", &State::new(function, value))?;
            }
        }

        property.end()
    }
}

impl<'de> de::Deserialize<'de> for Capability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_struct("Capability", FIELDS, CapabilityVisitor)
    }
}

struct CapabilityVisitor;

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum CapabilityField {
    Type,
    State,
}
const FIELDS: &[&str] = &["type", "state"];

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Value {
    String(String),
    Bool(bool),
    Float(f32),
}

impl<'de> de::Visitor<'de> for CapabilityVisitor {
    type Value = Capability;

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut cap_type: Option<String> = None;
        let mut state: Option<State<String, Value>> = None;

        while let Some(key) = map.next_key()? {
            match key {
                CapabilityField::Type => cap_type = Some(map.next_value()?),
                CapabilityField::State => state = Some(map.next_value()?),
            }
        }

        let cap_type = cap_type.ok_or_else(|| de::Error::missing_field("type"))?;
        let state = state.ok_or_else(|| de::Error::missing_field("state"))?;

        match cap_type.as_str() {
            "devices.capabilities.on_off" => {
                if let Value::Bool(value) = state.value {
                    Ok(Capability::OnOff { value })
                } else {
                    todo!()
                }
            }
            "devices.capabilities.mode" => {
                if let Value::String(value) = state.value {
                    let function =
                        ModeFunction::from_str(&state.instance).map_err(de::Error::custom)?;
                    let mode = Mode::from_str(&value).map_err(de::Error::custom)?;

                    Ok(Capability::Mode { function, mode })
                } else {
                    todo!()
                }
            }
            "devices.capabilities.toggle" => {
                if let Value::Bool(value) = state.value {
                    let function =
                        ToggleFunction::from_str(&state.instance).map_err(de::Error::custom)?;
                    Ok(Capability::Toggle { function, value })
                } else {
                    todo!()
                }
            }
            "devices.capabilities.range" => {
                if let Value::Float(value) = state.value {
                    let function =
                        RangeFunction::from_str(&state.instance).map_err(de::Error::custom)?;

                    Ok(Capability::Range {
                        function,
                        value,
                        relative: state.relative,
                    })
                } else {
                    todo!()
                }
            }
            _ => Err(de::Error::unknown_variant(
                &cap_type,
                &[
                    "devices.capabilities.on_off",
                    "devices.capabilities.mode",
                    "devices.capabilities.toggle",
                    "devices.capabilities.range",
                ],
            )),
        }
    }

    fn expecting(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn test_on_off() {
        assert_eq!(
            to_value(&Capability::OnOff { value: false }).unwrap(),
            json!({
                "type": "devices.capabilities.on_off",
                "state": {"instance": "on", "value": false}
            })
        );

        assert_eq!(
            from_value::<Capability>(json!({
                "type": "devices.capabilities.on_off",
                "state": {"instance": "on", "value": true}
            }))
            .unwrap(),
            Capability::OnOff { value: true }
        );
    }

    #[test]
    fn test_mode() {
        assert_eq!(
            to_value(&Capability::Mode {
                function: ModeFunction::WorkSpeed,
                mode: Mode::Medium
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.mode",
                "state": {"instance": "work_speed", "value": "medium"}
            })
        );

        assert_eq!(
            from_value::<Capability>(json!({
                "type": "devices.capabilities.mode",
                "state": {"instance": "work_speed", "value": "medium"}
            }))
            .unwrap(),
            Capability::Mode {
                function: ModeFunction::WorkSpeed,
                mode: Mode::Medium
            }
        );
    }

    #[test]
    fn test_toggle() {
        assert_eq!(
            to_value(&Capability::Toggle {
                function: ToggleFunction::Pause,
                value: false
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.toggle",
                "state": {"instance": "pause", "value": false}
            })
        );

        assert_eq!(
            from_value::<Capability>(json!({
                "type": "devices.capabilities.toggle",
                "state": {"instance": "pause", "value": true}
            }))
            .unwrap(),
            Capability::Toggle {
                function: ToggleFunction::Pause,
                value: true
            }
        );
    }

    #[test]
    fn test_range() {
        assert_eq!(
            to_value(&Capability::Range {
                function: RangeFunction::Temperature,
                value: 22.0,
                relative: false
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.range",
                "state": {"instance": "temperature", "value": 22.0}
            })
        );

        assert_eq!(
            from_value::<Capability>(json!({
                "type": "devices.capabilities.range",
                "state": {"instance": "temperature", "value": 23.0}
            }))
            .unwrap(),
            Capability::Range {
                function: RangeFunction::Temperature,
                value: 23.0,
                relative: false
            }
        );

        assert_eq!(
            from_value::<Capability>(json!({
                "type": "devices.capabilities.range",
                "state": {"instance": "temperature", "value": 2.0, "relative": true}
            }))
            .unwrap(),
            Capability::Range {
                function: RangeFunction::Temperature,
                value: 2.0,
                relative: true
            }
        );
    }
}
