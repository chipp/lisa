use std::str::FromStr;

use serde::de;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use crate::{Mode, ModeFunction};

#[derive(Debug, PartialEq)]
pub enum Capability {
    OnOff { value: bool },
    Mode { function: ModeFunction, mode: Mode },
}

impl Capability {
    pub fn on_off(value: bool) -> Capability {
        Capability::OnOff { value }
    }

    pub fn mode(function: ModeFunction, mode: Mode) -> Capability {
        Capability::Mode { function, mode }
    }
}

#[derive(Deserialize, Serialize)]
struct State<S, U> {
    instance: S,
    value: U,
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
                property.serialize_field(
                    "state",
                    &State {
                        instance: "on",
                        value,
                    },
                )?;
            }
            Capability::Mode { function, mode } => {
                property.serialize_field("type", "devices.capabilities.mode")?;
                property.serialize_field(
                    "state",
                    &State {
                        instance: function,
                        value: mode,
                    },
                )?;
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
            _ => Err(de::Error::invalid_value(
                de::Unexpected::Str(&cap_type),
                &"devices.capabilities.on_off or devices.capabilities.mode",
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
    fn test_capabilities() {
        assert_eq!(
            to_value(&Capability::OnOff { value: false }).unwrap(),
            json!({
                "type": "devices.capabilities.on_off",
                "state": {"instance": "on", "value": false}
            })
        );

        assert_eq!(
            to_value(&Capability::Mode {
                function: ModeFunction::CleanupMode,
                mode: Mode::Medium
            })
            .unwrap(),
            json!({
                "type": "devices.capabilities.mode",
                "state": {"instance": "cleanup_mode", "value": "medium"}
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

        assert_eq!(
            from_value::<Capability>(json!({
                "type": "devices.capabilities.mode",
                "state": {"instance": "cleanup_mode", "value": "medium"}
            }))
            .unwrap(),
            Capability::Mode {
                function: ModeFunction::CleanupMode,
                mode: Mode::Medium
            }
        );
    }
}
