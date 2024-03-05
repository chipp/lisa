use axum::{
    body::Body,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};
use log::{error, trace};

use super::token::{is_valid_token, TokenType};

pub enum ValidationError {
    Expired,
    NoToken,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response<Body> {
        match self {
            ValidationError::Expired => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    "WWW-Authenticate",
                    "Bearer error=\"invalid_token\" error_description=\"The access token has expired\"".parse().unwrap(),
                );

                (StatusCode::UNAUTHORIZED, headers).into_response()
            }
            ValidationError::NoToken => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    "WWW-Authenticate",
                    "Bearer error=\"invalid_token\" error_description=\"No access token has been provided\"".parse().unwrap(),
                );

                (StatusCode::UNAUTHORIZED, headers).into_response()
            }
        }
    }
}

pub fn validate_autorization(
    headers: &HeaderMap,
    request_name: &'static str,
) -> Result<(), ValidationError> {
    match extract_token_from_headers(&headers) {
        Some(token) if is_valid_token(token, TokenType::Access) => {
            trace!(target: request_name, "received a valid access token");
            Ok(())
        }
        Some(_) => {
            error!(
                target: request_name,
                "an expired access token has been provided"
            );

            Err(ValidationError::Expired)
        }
        None => Err(ValidationError::NoToken),
    }
}

const BEARER: &str = "Bearer ";
fn extract_token_from_headers(headers: &HeaderMap) -> Option<&str> {
    let authorization = headers.get("Authorization")?;
    let authorization = std::str::from_utf8(authorization.as_bytes()).ok()?;
    authorization.strip_prefix(BEARER)
}
