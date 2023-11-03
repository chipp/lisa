use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Status {
    pub battery_level: u8,
    pub is_enabled: bool,
    pub is_paused: bool,
    pub fan_speed: String,
    // TODO: add current rooms
}

impl From<xiaomi::Status> for Status {
    fn from(value: xiaomi::Status) -> Self {
        Status {
            battery_level: value.battery,
            is_enabled: value.state.is_enabled(),
            is_paused: value.state.is_paused(),
            fan_speed: value.fan_speed.to_string(),
        }
    }
}
