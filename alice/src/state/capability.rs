use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::{Mode, ModeFunction};

#[derive(Debug)]
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

impl serde::ser::Serialize for Capability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct State<S, U> {
            instance: S,
            value: U,
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

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
    }
}
