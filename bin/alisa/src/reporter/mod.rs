mod air_conditioner;
mod light;
mod recuperator;
mod temperature_sensor;
mod thermostat;
mod vacuum_cleaner;

pub use air_conditioner::prepare_air_conditioner_state;
pub use light::prepare_light_update;
pub use recuperator::{
    prepare_elizaveta_state, prepare_recuperator_current_state, prepare_recuperator_update,
};
pub use temperature_sensor::prepare_sensor_update;
pub use thermostat::{prepare_thermostat_current_state, prepare_thermostat_update};
pub use vacuum_cleaner::prepare_vacuum_updates;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use alice::{StateDevice, StateResponse};
use transport::elizabeth::State as ElizabethState;
use transport::state::StateUpdate;
use transport::{DeviceId, DeviceType};

use jiff::Timestamp;
use log::{debug, error, info, warn};
use tokio::sync::Notify;

#[derive(Default)]
struct Pending {
    devices: HashMap<DeviceId, StateDevice>,
    coalesced: usize,
}

impl Pending {
    fn insert(&mut self, device: StateDevice) {
        match self.devices.entry(device.id()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                self.coalesced += entry.get_mut().merge(device);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(device);
            }
        }
    }

    fn fields(&self) -> usize {
        self.devices.values().map(StateDevice::field_count).sum()
    }

    // New arrivals always win over values from a failed request.
    fn restore(&mut self, mut older: Pending) {
        let current = std::mem::take(self);
        older.coalesced += current.coalesced;
        for device in current.devices.into_values() {
            older.insert(device);
        }
        *self = older;
    }
}

#[derive(Default)]
struct Shared {
    pending: Mutex<Pending>,
    notify: Notify,
}

#[derive(Clone)]
pub struct Reporter {
    inner: reqwest::Client,
    skill_id: String,
    token: String,
    shared: Arc<Shared>,
}

impl Reporter {
    pub fn new(skill_id: String, token: String) -> Self {
        let inner = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()
            .expect("HTTP client");

        Self {
            inner,
            skill_id,
            token,
            shared: Arc::default(),
        }
    }

    pub fn report_update(&self, update: StateUpdate) {
        if let Some(devices) = device_from_update(update) {
            let mut pending = self.shared.pending.lock().expect("pending state lock");
            for device in devices {
                pending.insert(device);
            }
            debug!(
                "pending state fields: {}, coalesced: {}",
                pending.fields(),
                pending.coalesced
            );
            drop(pending);
            self.shared.notify.notify_one();
        }
    }

    pub async fn run(self) -> crate::Result<()> {
        let url = callback_url("https://dialogs.yandex.net/api/v1", &self.skill_id);
        self.run_at(url).await
    }

    async fn run_at(self, url: url::Url) -> crate::Result<()> {
        let mut backoff = Duration::from_secs(1);
        loop {
            let batch = {
                let mut pending = self.shared.pending.lock().expect("pending state lock");
                std::mem::take(&mut *pending)
            };
            if batch.devices.is_empty() {
                self.shared.notify.notified().await;
                continue;
            }

            let field_count = batch.fields();
            let coalesced = batch.coalesced;
            let body = StateResponse::notification_body(
                Timestamp::now().as_second(),
                "chipp",
                batch.devices.values().cloned().collect(),
            );
            debug!(
                "state update: {}",
                serde_json::to_string_pretty(&body).unwrap()
            );
            let start = Instant::now();
            let result = send_report(&self.inner, url.clone(), &self.token, &body).await;
            let (retry, retry_after) = match result {
                Ok(response) if response.status() == reqwest::StatusCode::ACCEPTED => {
                    info!(
                        "state update accepted: {field_count} fields, {coalesced} coalesced, {:?}",
                        start.elapsed()
                    );
                    (false, None)
                }
                Ok(response) => {
                    let status = response.status();
                    let retry_after = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        response
                            .headers()
                            .get(reqwest::header::RETRY_AFTER)
                            .and_then(|value| value.to_str().ok())
                            .and_then(parse_retry_after)
                    } else {
                        None
                    };
                    error!(
                        "unable to report state changes {status}, {:?}",
                        start.elapsed()
                    );
                    (
                        status == reqwest::StatusCode::TOO_MANY_REQUESTS
                            || status.is_server_error(),
                        retry_after,
                    )
                }
                Err(err) => {
                    error!(
                        "unable to report state changes {err}, {:?}",
                        start.elapsed()
                    );
                    (true, None)
                }
            };
            if retry {
                {
                    let mut pending = self.shared.pending.lock().expect("pending state lock");
                    pending.restore(batch);
                    warn!(
                        "retrying state update: {} pending fields, {} coalesced",
                        pending.fields(),
                        pending.coalesced
                    );
                }
                let delay = retry_after.map_or(backoff, |after| after.max(backoff));
                tokio::time::sleep(delay).await;
                backoff = (backoff * 2).min(Duration::from_secs(60));
            } else {
                backoff = Duration::from_secs(1);
            }
        }
    }
}

fn parse_retry_after(value: &str) -> Option<Duration> {
    if let Ok(seconds) = value.trim().parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let date = jiff::fmt::rfc2822::parse(value).ok()?.timestamp();
    Some(Duration::try_from(date.duration_since(Timestamp::now())).unwrap_or_default())
}

async fn send_report<T: serde::Serialize>(
    client: &reqwest::Client,
    url: url::Url,
    token: &str,
    body: &T,
) -> std::result::Result<reqwest::Response, reqwest::Error> {
    client
        .post(url)
        .header(reqwest::header::AUTHORIZATION, format!("OAuth {token}"))
        .json(body)
        .send()
        .await
}

fn callback_url(base: &str, skill_id: &str) -> url::Url {
    let mut url = url::Url::parse(base).unwrap();
    url.path_segments_mut()
        .unwrap()
        .pop_if_empty()
        .extend(["skills", skill_id, "callback", "state"]);
    url
}

fn device_from_update(update: StateUpdate) -> Option<Vec<StateDevice>> {
    match update {
        StateUpdate::Elizaveta(state) => Some(vec![prepare_elizaveta_state(state)]),
        StateUpdate::Elizabeth(state) => Some(vec![prepare_elizabeth_device(state)?]),
        StateUpdate::Elisa(state) => Some(prepare_vacuum_updates(state)),
        StateUpdate::Isabel(state) => Some(vec![prepare_sensor_update(state)]),
        StateUpdate::Elisheba(state) => Some(vec![prepare_light_update(state)]),
        StateUpdate::Elzhbieta(state) => Some(vec![prepare_air_conditioner_state(state)]),
    }
}

fn prepare_elizabeth_device(state: ElizabethState) -> Option<StateDevice> {
    match state.device_type {
        DeviceType::Recuperator => prepare_recuperator_update(state),
        DeviceType::Thermostat => prepare_thermostat_update(state),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;
    fn server(status: &str, body: &str) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nLocation: /redirected\r\nConnection: close\r\n\r\n{body}", body.len());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 1024];
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buf[..n]);
                if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..pos]);
                    let len = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= pos + 4 + len {
                        break;
                    }
                }
            }
            stream.write_all(response.as_bytes()).unwrap();
            String::from_utf8(bytes).unwrap()
        });
        (format!("http://{address}"), handle)
    }

    #[tokio::test]
    async fn sends_authenticated_report_and_does_not_follow_redirects() {
        let (base, handle) = server("302 Found", "failure");
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let response = send_report(
            &client,
            callback_url(&format!("{base}/api/v1"), "a/b"),
            "secret",
            &serde_json::json!({"state": 1}),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::FOUND);
        let request = handle.join().unwrap();
        assert!(request.starts_with("POST /api/v1/skills/a%2Fb/callback/state HTTP/1.1\r\n"));
        let request = request.to_lowercase();
        assert!(request.contains("authorization: oauth secret"));
        assert!(request.contains("content-type: application/json"));
        assert!(request.ends_with("\r\n\r\n{\"state\":1}"));
    }
}

#[cfg(test)]
mod pending_tests {
    use super::*;
    use alice::{StateCapability, StateProperty};
    use transport::Room;

    fn sensor(properties: Vec<StateProperty>) -> StateDevice {
        StateDevice::new_with_properties(
            DeviceId::temperature_sensor_at_room(Room::HomeOffice),
            properties,
        )
    }

    #[test]
    fn merges_fields_and_restores_without_overwriting_newer_values() {
        let mut sent = Pending::default();
        sent.insert(sensor(vec![
            StateProperty::humidity(56.0),
            StateProperty::temperature(20.0),
        ]));
        let mut pending = Pending::default();
        pending.insert(sensor(vec![StateProperty::humidity(58.0)]));
        pending.insert(sensor(vec![StateProperty::humidity(61.0)]));
        pending.restore(sent);
        assert_eq!(pending.fields(), 2);
        let value = serde_json::to_value(pending.devices.values().next().unwrap()).unwrap();
        let properties = value["properties"].as_array().unwrap();
        assert!(properties
            .iter()
            .any(|p| p["state"]["instance"] == "humidity" && p["state"]["value"] == 61.0));
        assert!(properties
            .iter()
            .any(|p| p["state"]["instance"] == "temperature" && p["state"]["value"] == 20.0));
        assert_eq!(pending.coalesced, 2);
    }

    #[test]
    fn merging_capabilities_preserves_distinct_functions() {
        let id = DeviceId::temperature_sensor_at_room(Room::HomeOffice);
        let mut state =
            StateDevice::new_with_capabilities(id, vec![StateCapability::on_off(false)]);
        state.merge(StateDevice::new_with_capabilities(
            id,
            vec![StateCapability::on_off(true)],
        ));
        assert_eq!(state.field_count(), 1);
        let value = serde_json::to_value(state).unwrap();
        assert_eq!(value["capabilities"][0]["state"]["value"], true);
    }

    #[test]
    fn retry_after_supports_seconds_and_http_dates() {
        assert_eq!(parse_retry_after(" 12 "), Some(Duration::from_secs(12)));
        assert_eq!(
            parse_retry_after("Wed, 21 Oct 2015 07:28:00 GMT"),
            Some(Duration::ZERO)
        );
        assert_eq!(parse_retry_after("invalid"), None);
    }

    #[tokio::test]
    async fn retry_sends_latest_values_received_during_request() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap())
            .parse()
            .unwrap();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let mut requests = Vec::new();
            let mut started_tx = Some(started_tx);
            for n in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut buffer = [0; 2048];
                loop {
                    let count = stream.read(&mut buffer).unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&buffer[..count]);
                    if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..pos]);
                        let length = headers
                            .lines()
                            .find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().unwrap())
                            })
                            .unwrap();
                        if bytes.len() >= pos + 4 + length {
                            break;
                        }
                    }
                }
                requests.push(String::from_utf8(bytes).unwrap());
                if n == 0 {
                    started_tx.take().unwrap().send(()).ok();
                    release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                    stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                } else {
                    stream.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                }
            }
            requests
        });
        let reporter = Reporter::new("skill".into(), "secret".into());
        {
            let mut pending = reporter.shared.pending.lock().unwrap();
            pending.insert(sensor(vec![
                StateProperty::humidity(56.0),
                StateProperty::temperature(20.0),
            ]));
        }
        let worker = tokio::spawn(reporter.clone().run_at(url));
        started_rx.await.unwrap();
        {
            let mut pending = reporter.shared.pending.lock().unwrap();
            pending.insert(sensor(vec![StateProperty::humidity(61.0)]));
        }
        reporter.shared.notify.notify_one();
        release_tx.send(()).unwrap();
        let requests = tokio::task::spawn_blocking(|| server.join().unwrap())
            .await
            .unwrap();
        worker.abort();
        let body = requests[1].split("\r\n\r\n").nth(1).unwrap();
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        let properties = body["payload"]["devices"][0]["properties"]
            .as_array()
            .unwrap();
        assert!(properties
            .iter()
            .any(|p| p["state"]["instance"] == "humidity" && p["state"]["value"] == 61.0));
        assert!(properties
            .iter()
            .any(|p| p["state"]["instance"] == "temperature" && p["state"]["value"] == 20.0));
    }
}
