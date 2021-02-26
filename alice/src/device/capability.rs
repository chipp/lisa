use serde::ser::SerializeStruct;
use serde::Serialize;

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
        let mut property = serializer.serialize_struct("Property", 4)?;

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
                struct Parameters<'a> {
                    instance: &'a ModeFunction,
                    modes: &'a [Mode],
                }

                property.serialize_field("type", "devices.capabilities.mode")?;
                property.serialize_field("retreivable", retreivable)?;
                property.serialize_field("reportable", reportable)?;
                property.serialize_field(
                    "parameters",
                    &Parameters {
                        instance: function,
                        modes,
                    },
                )?;
            }
        }

        property.end()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeFunction {
    FanSpeed,
}

#[derive(Debug, Serialize)]
#[serde(tag = "value", rename_all = "snake_case")]
pub enum Mode {
    Quiet,
    Medium,
    High,
    Turbo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn test_modes() {
        assert_eq!(to_value(&Mode::Quiet).unwrap(), json!({"value": "quiet"}));
        assert_eq!(to_value(&Mode::Medium).unwrap(), json!({"value": "medium"}));
        assert_eq!(to_value(&Mode::High).unwrap(), json!({"value": "high"}));
        assert_eq!(to_value(&Mode::Turbo).unwrap(), json!({"value": "turbo"}));
    }

    #[test]
    fn test_mode_functions() {
        assert_eq!(
            to_value(&ModeFunction::FanSpeed).unwrap(),
            json!("fan_speed")
        );
    }

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
