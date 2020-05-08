use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    #[serde(rename = "response")]
    pub payload: ResponsePayload,
    pub version: String,
}

#[derive(Serialize)]
pub struct ResponsePayload {
    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<String>,

    pub end_session: bool,
}

impl Response {
    pub fn from_string<T: ToString>(text: T) -> Response {
        let payload = ResponsePayload {
            text: text.to_string(),
            tts: None,
            end_session: true,
        };

        Response {
            payload,
            version: "1.0".to_string(),
        }
    }
}
