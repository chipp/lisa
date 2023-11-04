use crate::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use log::{error, trace};

use super::token::{is_valid_token, TokenType};

pub async fn validate_autorization<F, T>(
    request: Request<Body>,
    request_name: &'static str,
    success: F,
) -> Result<Response<Body>>
where
    F: FnOnce(Request<Body>) -> T,
    T: std::future::Future<Output = Result<Response<Body>>>,
{
    match extract_token_from_headers(request.headers()) {
        Some(token) if is_valid_token(token, TokenType::Access) => {
            trace!(target: request_name, "received a valid access token");
            success(request).await
        }
        Some(_) => {
            error!(
                target: request_name,
                "an expired access token has been provided"
            );

            let response = Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(
                    "WWW-Authenticate",
                    r#"Bearer error="invalid_token" error_description="The access token has expired""#,
                )
                .body(Body::from("invalid token"))
                .unwrap();

            Ok(response)
        }
        None => {
            let response = Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header("WWW-Authenticate", r#"Bearer error="invalid_token" error_description="No access token has been provided""#)
                        .body(Body::from("invalid token"))?;

            Ok(response)
        }
    }
}

const BEARER: &str = "Bearer ";
fn extract_token_from_headers(headers: &header::HeaderMap) -> Option<&str> {
    let authorization = headers.get("Authorization")?;
    let authorization = std::str::from_utf8(authorization.as_bytes()).ok()?;

    if authorization.starts_with(BEARER) {
        Some(&authorization[BEARER.len()..])
    } else {
        None
    }
}
