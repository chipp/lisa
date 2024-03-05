use axum::{http::StatusCode, response::IntoResponse};

pub async fn pong() -> impl IntoResponse {
    StatusCode::OK
}
