use std::net::Ipv4Addr;
use std::time::Duration;

use log::{info, warn};

use crate::local::TcpLocalConnection;
use crate::protocol::DecodeError;
use crate::util::Counter;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanSpeed {
    Off,
    Silent,
    Standard,
    Medium,
    Turbo,
    Max,
    SmartMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupMode {
    DryCleaning,
    WetCleaning,
    MixedCleaning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MopMode {
    Level1,
    Level2,
    Level3,
    Unknown(i64),
}

impl MopMode {
    fn from_code(code: i64) -> Self {
        match code {
            300 => MopMode::Level1,
            301 => MopMode::Level2,
            302 => MopMode::Level3,
            _ => MopMode::Unknown(code),
        }
    }

    pub fn code(self) -> i64 {
        match self {
            MopMode::Level1 => 300,
            MopMode::Level2 => 301,
            MopMode::Level3 => 302,
            MopMode::Unknown(code) => code,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterBoxMode {
    Off,
    Low,
    Medium,
    High,
    Max,
    Custom,
    SmartMode,
    Unknown(i64),
}

impl WaterBoxMode {
    fn from_code(code: i64) -> Self {
        match code {
            200 => WaterBoxMode::Off,
            201 => WaterBoxMode::Low,
            202 => WaterBoxMode::Medium,
            203 => WaterBoxMode::High,
            208 => WaterBoxMode::Max,
            204 => WaterBoxMode::Custom,
            209 => WaterBoxMode::SmartMode,
            _ => WaterBoxMode::Unknown(code),
        }
    }

    pub fn code(self) -> i64 {
        match self {
            WaterBoxMode::Off => 200,
            WaterBoxMode::Low => 201,
            WaterBoxMode::Medium => 202,
            WaterBoxMode::High => 203,
            WaterBoxMode::Max => 208,
            WaterBoxMode::Custom => 204,
            WaterBoxMode::SmartMode => 209,
            WaterBoxMode::Unknown(code) => code,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WashStatus {
    Idle,
    Active(i64),
}

impl WashStatus {
    fn from_code(code: i64) -> Self {
        if code == 0 {
            WashStatus::Idle
        } else {
            WashStatus::Active(code)
        }
    }

    pub fn code(self) -> i64 {
        match self {
            WashStatus::Idle => 0,
            WashStatus::Active(code) => code,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WashPhase {
    Idle,
    Unknown(i64),
}

impl WashPhase {
    fn from_code(code: i64) -> Self {
        match code {
            0 => WashPhase::Idle,
            _ => WashPhase::Unknown(code),
        }
    }

    pub fn code(self) -> i64 {
        match self {
            WashPhase::Idle => 0,
            WashPhase::Unknown(code) => code,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Unknown,
    Idle,
    Cleaning,
    Returning,
    Docked,
    Paused,
}

impl State {
    pub fn is_enabled(&self) -> bool {
        matches!(self, State::Cleaning)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, State::Paused | State::Idle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub battery: u8,
    pub state: State,
    pub fan_speed: FanSpeed,
    pub cleanup_mode: CleanupMode,
    pub error_code: i64,
    pub dock_error_status: i64,
    pub dust_collection_status: i64,
    pub auto_dust_collection: i64,
    pub water_box_status: i64,
    pub water_box_mode: WaterBoxMode,
    pub mop_mode: MopMode,
    pub wash_status: WashStatus,
    pub wash_phase: WashPhase,
    pub water_shortage_status: i64,
    pub clean_area: i64,
    pub clean_time: i64,
    pub clean_percent: i64,
}

pub struct Vacuum {
    ip: Ipv4Addr,
    duid: String,
    local_key: String,
    connection: TcpLocalConnection,
    last_cleaning_rooms: Vec<u8>,
    id_counter: Counter,
    seq_counter: Counter,
    random_counter: Counter,
}

impl Vacuum {
    pub async fn new(ip: Ipv4Addr, duid: String, local_key: String) -> Result<Vacuum> {
        let mut id_counter = Counter::new(10_000, 32_767);
        let mut seq_counter = Counter::new(100_000, 999_999);
        let random_counter = Counter::new(10_000, 99_999);

        let nonce = id_counter.next();
        let connection =
            TcpLocalConnection::connect(ip, local_key.clone(), nonce, seq_counter.next(), nonce)
                .await?;
        info!(
            "roborock connected (duid={}, protocol={:?})",
            duid,
            connection.protocol_version()
        );
        Ok(Vacuum {
            ip,
            duid,
            local_key,
            connection,
            last_cleaning_rooms: vec![],
            id_counter,
            seq_counter,
            random_counter,
        })
    }

    pub fn last_cleaning_rooms(&self) -> &[u8] {
        &self.last_cleaning_rooms
    }

    pub async fn status(&mut self) -> Result<Status> {
        let result = self
            .send_rpc_with_retry("get_status", serde_json::json!([]))
            .await?;
        let status_value = status_from_result(result);
        let battery = status_value
            .get("battery")
            .and_then(|value| value.as_u64())
            .unwrap_or(0) as u8;
        let fan_code = get_i64(&status_value, "fan_power");

        Ok(Status {
            battery,
            state: state_from_status(&status_value),
            fan_speed: fan_from_code(fan_code),
            cleanup_mode: cleanup_mode_from_status(&status_value, fan_code),
            error_code: get_i64(&status_value, "error_code"),
            dock_error_status: get_i64(&status_value, "dock_error_status"),
            dust_collection_status: get_i64(&status_value, "dust_collection_status"),
            auto_dust_collection: get_i64(&status_value, "auto_dust_collection"),
            water_box_status: get_i64(&status_value, "water_box_status"),
            water_box_mode: water_box_mode_from_status(&status_value),
            mop_mode: MopMode::from_code(get_i64(&status_value, "mop_mode")),
            wash_status: WashStatus::from_code(get_i64(&status_value, "wash_status")),
            wash_phase: WashPhase::from_code(get_i64(&status_value, "wash_phase")),
            water_shortage_status: get_i64(&status_value, "water_shortage_status"),
            clean_area: get_i64(&status_value, "clean_area"),
            clean_time: get_i64(&status_value, "clean_time"),
            clean_percent: get_i64(&status_value, "clean_percent"),
        })
    }

    pub async fn set_fan_speed(&mut self, fan_speed: FanSpeed) -> Result<()> {
        let code = fan_to_code(fan_speed);
        self.send_rpc_with_retry("set_custom_mode", serde_json::json!([code]))
            .await?;
        Ok(())
    }

    pub async fn set_cleanup_mode(&mut self, cleanup_mode: CleanupMode) -> Result<()> {
        let water_box_mode = cleanup_mode_to_water_box_mode(cleanup_mode);
        let params = serde_json::json!({ "water_box_mode": water_box_mode.code() });
        self.send_rpc_with_retry("set_water_box_custom_mode", params)
            .await?;
        if cleanup_mode == CleanupMode::WetCleaning {
            self.send_rpc_with_retry("set_custom_mode", serde_json::json!([105]))
                .await?;
        } else {
            let result = self
                .send_rpc_with_retry("get_status", serde_json::json!([]))
                .await?;
            let status_value = status_from_result(result);
            let fan_code = get_i64(&status_value, "fan_power");
            if fan_code == 105 {
                self.set_fan_speed(FanSpeed::Standard).await?;
            }
        }
        Ok(())
    }

    pub async fn start(&mut self, room_ids: Vec<u8>) -> Result<()> {
        self.last_cleaning_rooms = room_ids.clone();
        if room_ids.is_empty() {
            self.send_rpc_with_retry("app_start", serde_json::json!([]))
                .await?;
            return Ok(());
        }
        let params = serde_json::json!([{ "segments": room_ids, "repeat": 1 }]);
        self.send_rpc_with_retry("app_segment_clean", params)
            .await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.send_rpc_with_retry("app_stop", serde_json::json!([]))
            .await?;
        Ok(())
    }

    pub async fn go_home(&mut self) -> Result<()> {
        self.send_rpc_with_retry("app_charge", serde_json::json!([]))
            .await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        self.send_rpc_with_retry("app_pause", serde_json::json!([]))
            .await?;
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        self.send_rpc_with_retry("app_start", serde_json::json!([]))
            .await?;
        Ok(())
    }

    async fn send_rpc_with_retry(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        const RETRY_DELAY: Duration = Duration::from_millis(300);
        let request_id = self.id_counter.next();
        let mut attempt = 0;
        loop {
            match self
                .connection
                .send_rpc(
                    request_id,
                    self.seq_counter.next(),
                    self.random_counter.next(),
                    method,
                    params.clone(),
                )
                .await
            {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if attempt >= 1 || !should_retry(err.as_ref()) {
                        return Err(err);
                    }
                    let next_attempt = attempt + 1;
                    warn!(
                        "roborock rpc failed (method={}, attempt={}): {}, reconnecting",
                        method, next_attempt, err
                    );
                    self.reconnect().await?;
                    tokio::time::sleep(RETRY_DELAY).await;
                    attempt = next_attempt;
                }
            }
        }
    }

    async fn reconnect(&mut self) -> Result<()> {
        let nonce = self.id_counter.next();
        let connection = TcpLocalConnection::connect(
            self.ip,
            self.local_key.clone(),
            nonce,
            self.seq_counter.next(),
            nonce,
        )
        .await?;
        info!(
            "roborock reconnected (duid={}, protocol={:?})",
            self.duid,
            connection.protocol_version()
        );
        self.connection = connection;
        Ok(())
    }
}

fn state_from_code(code: i64) -> State {
    match code {
        1 | 4 | 5 | 7 | 11 | 16 | 17 | 18 | 22 | 23 | 25 | 29 | 6301..=6309 => State::Cleaning,
        2 | 3 => State::Idle,
        6 | 15 | 26 => State::Returning,
        8 | 9 | 100 => State::Docked,
        10 => State::Paused,
        _ => State::Unknown,
    }
}

fn status_from_result(result: serde_json::Value) -> serde_json::Value {
    match result {
        serde_json::Value::Array(mut values) => values
            .drain(..)
            .next()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
        _ => result,
    }
}

fn get_i64(status: &serde_json::Value, key: &str) -> i64 {
    status
        .get(key)
        .and_then(|value| value.as_i64())
        .unwrap_or(0)
}

fn water_box_mode_from_status(status: &serde_json::Value) -> WaterBoxMode {
    WaterBoxMode::from_code(get_i64(status, "water_box_mode"))
}

fn cleanup_mode_from_status(status: &serde_json::Value, fan_code: i64) -> CleanupMode {
    match water_box_mode_from_status(status) {
        WaterBoxMode::Off => CleanupMode::DryCleaning,
        _ => {
            if fan_code == 105 {
                CleanupMode::WetCleaning
            } else {
                CleanupMode::MixedCleaning
            }
        }
    }
}

fn cleanup_mode_to_water_box_mode(mode: CleanupMode) -> WaterBoxMode {
    match mode {
        CleanupMode::DryCleaning => WaterBoxMode::Off,
        CleanupMode::WetCleaning => WaterBoxMode::Max,
        CleanupMode::MixedCleaning => WaterBoxMode::Medium,
    }
}

fn state_from_status(status: &serde_json::Value) -> State {
    let state_code = status.get("state").and_then(|value| value.as_i64());
    if let Some(code) = state_code {
        let mapped = state_from_code(code);
        if mapped != State::Unknown {
            return mapped;
        }
    }

    let in_cleaning = status.get("in_cleaning").and_then(|value| value.as_i64());
    if in_cleaning == Some(1) {
        return State::Cleaning;
    }
    let in_returning = status.get("in_returning").and_then(|value| value.as_i64());
    if in_returning == Some(1) {
        return State::Returning;
    }
    let charge_status = status.get("charge_status").and_then(|value| value.as_i64());
    if charge_status == Some(1) {
        return State::Docked;
    }
    State::Unknown
}

fn fan_from_code(code: i64) -> FanSpeed {
    match code {
        38 | 50 | 101 | 0 => FanSpeed::Silent,
        60 | 68 | 75 | 77 | 102 | 1 => FanSpeed::Standard,
        90 | 100 | 103 | 2 => FanSpeed::Medium,
        104 | 3 => FanSpeed::Max,
        105 => FanSpeed::Off,
        106 => FanSpeed::Standard,
        108 => FanSpeed::Max,
        110 => FanSpeed::SmartMode,
        _ => FanSpeed::Standard,
    }
}

fn fan_to_code(speed: FanSpeed) -> i64 {
    match speed {
        FanSpeed::Off => 105,
        FanSpeed::Silent => 101,
        FanSpeed::Standard => 102,
        FanSpeed::Medium => 103,
        FanSpeed::Turbo => 104,
        FanSpeed::Max => 104,
        FanSpeed::SmartMode => 110,
    }
}

fn should_retry(err: &(dyn std::error::Error + 'static)) -> bool {
    if err.downcast_ref::<DecodeError>().is_some() {
        return true;
    }
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        use std::io::ErrorKind;
        return matches!(
            io_err.kind(),
            ErrorKind::BrokenPipe
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::NotConnected
                | ErrorKind::UnexpectedEof
                | ErrorKind::TimedOut
        );
    }
    let message = err.to_string();
    message.contains("connection closed")
        || message.contains("Broken pipe")
        || message.contains("deadline has elapsed")
}
