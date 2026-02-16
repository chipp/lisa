use std::time::{SystemTime, UNIX_EPOCH};

use crypto::Token;

use base64::prelude::*;
use rand::Rng;
use serde::Serialize;

#[derive(Serialize)]
pub struct RequestBody<'d> {
    sequence: String,
    iv: String,
    data: String,
    #[serde(rename = "selfApikey")]
    self_apikey: String,
    #[serde(rename = "deviceid")]
    device_id: &'d str,
    encrypt: bool,
}

#[derive(Serialize)]
struct Data {
    switches: [Switch; 1],
}

#[derive(Serialize)]
struct Switch {
    switch: &'static str,
    outlet: u8,
}

impl RequestBody<'_> {
    pub fn new(is_enabled: bool, device_id: &str, key: Token<16>) -> RequestBody<'_> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = timestamp.as_millis().to_string();

        let data = Data {
            switches: [Switch {
                switch: if is_enabled { "on" } else { "off" },
                outlet: 0,
            }],
        };
        let mut data = serde_json::to_vec(&data).unwrap();

        let iv = generate_iv();
        let data = crypto::cbc::encrypt(&mut data, key, iv).unwrap();

        RequestBody {
            sequence: timestamp,
            iv: BASE64_STANDARD.encode(iv),
            data: BASE64_STANDARD.encode(data),
            self_apikey: "123".to_string(),
            device_id,
            encrypt: true,
        }
    }
}

fn generate_iv() -> Token<16> {
    let mut iv = [0u8; 16];
    rand::rng().fill_bytes(&mut iv);
    iv
}
