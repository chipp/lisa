use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::{Mode, ModeFunction};

#[derive(Debug)]
pub enum Capability {
    OnOff {
        split: bool,
        retreivable: bool,
        reportable: bool,
    },
    Mode {
        function: ModeFunction,
        modes: Vec<Mode>,
        retreivable: bool,
        reportable: bool,
    },
}

impl Capability {
    pub fn on_off(split: bool) -> Capability {
        Capability::OnOff {
            split,
            retreivable: false,
            reportable: false,
        }
    }

    pub fn mode(function: ModeFunction, modes: Vec<Mode>) -> Capability {
        Capability::Mode {
            function,
            modes,
            retreivable: false,
            reportable: false,
        }
    }

    pub fn retrievable(self) -> Capability {
        let mut value = self;

        match value {
            Capability::OnOff {
                split: _,
                ref mut retreivable,
                reportable: _,
            } => *retreivable = true,
            Capability::Mode {
                function: _,
                modes: _,
                ref mut retreivable,
                reportable: _,
            } => *retreivable = true,
        }

        value
    }

    pub fn reportable(self) -> Capability {
        let mut value = self;

        match value {
            Capability::OnOff {
                split: _,
                retreivable: _,
                ref mut reportable,
            } => *reportable = true,
            Capability::Mode {
                function: _,
                modes: _,
                retreivable: _,
                ref mut reportable,
            } => *reportable = true,
        }

        value
    }
}

impl serde::ser::Serialize for Capability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut property = serializer.serialize_struct("Capability", 4)?;

        match self {
            Capability::OnOff {
                split,
                retreivable,
                reportable,
            } => {
                #[derive(Serialize)]
                struct Parameters<'a> {
                    split: &'a bool,
                }

                property.serialize_field("type", "devices.capabilities.on_off")?;
                property.serialize_field("retreivable", retreivable)?;
                property.serialize_field("reportable", reportable)?;
                property.serialize_field("parameters", &Parameters { split })?;
            }
            Capability::Mode {
                function,
                modes,
                retreivable,
                reportable,
            } => {
                #[derive(Serialize)]
                struct ModeWrapper<'a> {
                    value: &'a Mode,
                }

                #[derive(Serialize)]
                struct Parameters<'a> {
                    instance: &'a ModeFunction,
                    modes: Vec<ModeWrapper<'a>>,
                }

                property.serialize_field("type", "devices.capabilities.mode")?;
                property.serialize_field("retreivable", retreivable)?;
                property.serialize_field("reportable", reportable)?;
                property.serialize_field(
                    "parameters",
                    &Parameters {
                        instance: function,
                        modes: modes.iter().map(|m| ModeWrapper { value: m }).collect(),
                    },
                )?;
            }
        }

        property.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Mode, ModeFunction};

    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn test_on_off_capability() {
        let capability = Capability::OnOff {
            split: false,
            reportable: true,
            retreivable: false,
        };

        assert_eq!(
            to_value(&capability).unwrap(),
            json!({
                "type": "devices.capabilities.on_off",
                "reportable": true,
                "retreivable": false,
                "parameters": {
                    "split": false
                }
            })
        );
    }

    #[test]
    fn test_mode_capability() {
        let capability = Capability::Mode {
            function: ModeFunction::FanSpeed,
            modes: vec![Mode::Quiet, Mode::High],
            reportable: true,
            retreivable: false,
        };

        assert_eq!(
            to_value(&capability).unwrap(),
            json!({
                "type": "devices.capabilities.mode",
                "reportable": true,
                "retreivable": false,
                "parameters": {
                    "instance": "fan_speed",
                    "modes": [
                        {"value": "quiet"},
                        {"value": "high"}
                    ]
                }
            })
        );
    }
}
