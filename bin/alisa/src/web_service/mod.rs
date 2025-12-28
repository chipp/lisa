mod auth {
    mod auth_page;
    mod authorize;
    mod issue_token;
    mod token;
    mod validate_autorization;

    pub use auth_page::auth_page;
    pub use authorize::authorize;
    pub use issue_token::issue_token;
    pub use validate_autorization::validate_autorization;
}

mod user {
    mod action;
    mod devices;
    mod pong;
    mod query;
    mod unlink;

    pub use action::action;
    pub use devices::devices;
    pub use pong::pong;
    pub use query::query;
    pub use unlink::unlink;
}

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, head, post};
use axum::Router;
use log::error;

use crate::Error;

pub struct ServiceError(Error, uuid::Uuid);

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response<Body> {
        error!("ServiceError[{}]: {}", self.1, self.0);

        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl From<Error> for ServiceError {
    fn from(value: Error) -> Self {
        ServiceError(value, uuid::Uuid::new_v4())
    }
}

impl From<paho_mqtt::Error> for ServiceError {
    fn from(value: paho_mqtt::Error) -> Self {
        ServiceError(Error::Mqtt(value), uuid::Uuid::new_v4())
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(value: serde_json::Error) -> Self {
        ServiceError(Error::Json(value), uuid::Uuid::new_v4())
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/auth", get(auth::auth_page).post(auth::authorize))
        .route("/token", post(auth::issue_token))
        .route("/v1.0", head(user::pong).get(user::pong))
        .route("/v1.0/user/devices", get(user::devices))
        .route("/v1.0/user/devices/query", post(user::query))
        .route("/v1.0/user/devices/action", post(user::action))
        .route("/v1.0/user/unlink", post(user::unlink))
}
