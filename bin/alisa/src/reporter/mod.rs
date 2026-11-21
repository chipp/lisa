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

use crate::Result;
use alice::{StateDevice, StateResponse};
use transport::elizabeth::State as ElizabethState;
use transport::state::StateUpdate;
use transport::DeviceType;

use jiff::Timestamp;
use log::{debug, error};

pub struct Reporter {
    inner: reqwest::Client,
    skill_id: String,
    token: String,
}

impl Reporter {
    pub fn new(skill_id: String, token: String) -> Self {
        let inner = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("HTTP client");

        Self {
            inner,
            skill_id,
            token,
        }
    }

    pub async fn report_update(&self, update: StateUpdate) -> Result<()> {
        let devices = if let Some(devices) = device_from_update(update) {
            devices
        } else {
            return Ok(());
        };

        let now = Timestamp::now();
        let body = StateResponse::notification_body(now.as_second(), "chipp", devices);

        debug!(
            "state update: {}",
            serde_json::to_string_pretty(&body).unwrap()
        );

        let url = callback_url("https://dialogs.yandex.net/api/v1", &self.skill_id);
        let result = send_report(&self.inner, url, &self.token, &body).await;

        match result {
            Ok(response) if response.status() == reqwest::StatusCode::ACCEPTED => {
                debug!("successfully notified alice about changes");
            }
            Ok(response) => {
                error!("unable to report state changes {}", response.status());
                debug!("{response:#?}");
                match response.bytes().await {
                    Ok(body) => match std::str::from_utf8(&body) {
                        Ok(json) => debug!("{}", json),
                        Err(err) => error!("unable to report state changes {}", err),
                    },
                    Err(err) => error!("unable to report state changes {}", err),
                }
            }
            Err(err) => error!("unable to report state changes {}", err),
        }

        Ok(())
    }
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
