use serde::Deserialize;
use std::collections::HashMap;

mod payload;

pub use payload::RequestPayload;

#[derive(Deserialize)]
pub struct Request {
    pub meta: RequestMeta,
    #[serde(rename = "request")]
    pub payload: RequestPayload,
    pub session: RequestSession,
}

#[derive(Deserialize)]
pub struct RequestMeta {
    pub locale: String,
    pub timezone: String,
    pub client_id: String,
    pub interfaces: HashMap<String, HashMap<String, ()>>,
}

#[derive(Deserialize)]
pub struct RequestSession {
    pub session_id: String,
    pub message_id: String,
    pub skill_id: String,

    pub application: RequestApplication,
    pub new: bool,
}

#[derive(Deserialize)]
pub struct RequestApplication {
    #[serde(rename = "application_id")]
    pub id: String,
}
