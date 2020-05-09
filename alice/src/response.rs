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
    pub end_session: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub buttons: Vec<ResponseButton>,
}

impl Response {
    pub fn from_string<T: ToString>(text: T) -> Response {
        let payload = ResponsePayload {
            text: text.to_string(),
            end_session: true,
            tts: None,
            buttons: vec![],
        };

        Response {
            payload,
            version: "1.0".to_string(),
        }
    }

    pub fn add_button(&mut self, button: ResponseButton) {
        self.payload.buttons.push(button)
    }
}

#[derive(Serialize)]
pub struct ResponseButton {
    pub title: String,
    pub hide: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl ResponseButton {
    pub fn with_title<S: ToString>(title: S) -> ResponseButton {
        ResponseButton {
            title: title.to_string(),
            hide: true,
            url: None,
            payload: None,
        }
    }

    pub fn set_hide(mut self, hide: bool) -> ResponseButton {
        self.hide = hide;
        self
    }
}
