use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeFunction {
    FanSpeed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Quiet,
    Medium,
    High,
    Turbo,
}
