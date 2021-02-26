use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Capability {
    #[serde(rename = "devices.capabilities.on_off")]
    OnOff {
        reportable: bool,
        retreivable: bool,
        parameters: OnOffParameters,
    },

    #[serde(rename = "devices.capabilities.mode")]
    Mode {
        reportable: bool,
        retreivable: bool,
        parameters: ModeParameters,
    },
}

#[derive(Debug, Serialize)]
pub struct OnOffParameters {
    pub split: bool,
}

#[derive(Debug, Serialize)]
pub struct ModeParameters {
    pub instance: ModeFunction,
    pub modes: Vec<Mode>,
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
    fn test_mode_parameters() {
        assert_eq!(
            to_value(&ModeParameters {
                instance: ModeFunction::FanSpeed,
                modes: vec![Mode::Quiet, Mode::High],
            })
            .unwrap(),
            json!({
                "instance": "fan_speed",
                "modes": [
                    {"value": "quiet"},
                    {"value": "high"}
                ]
            })
        );
    }

    #[test]
    fn test_on_off_capability() {
        let capability = Capability::OnOff {
            reportable: true,
            retreivable: false,
            parameters: OnOffParameters { split: false },
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
            reportable: true,
            retreivable: false,
            parameters: ModeParameters {
                instance: ModeFunction::FanSpeed,
                modes: vec![Mode::Quiet, Mode::High],
            },
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
