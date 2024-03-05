use std::borrow::Cow;

use chrono::Duration;
use hyper::{body::HttpBody, Body, Request, Response, StatusCode};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_urlencoded::de;

use crate::Result;

use super::token::{create_token_with_expiration_in, is_valid_token, TokenType};

pub async fn issue_token(request: Request<Body>) -> Result<Response<Body>> {
    let body = request.into_body().collect().await?.to_bytes();
    let client_creds: ClientCreds = de::from_bytes(&body).unwrap();

    if !validate_client_creds(&client_creds) {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("invalid client creds"))?);
    }

    match client_creds.grant_type {
        GrantType::AuthorizationCode => {
            let auth_code: AuthorizationCode = de::from_bytes(&body).unwrap();

            if is_valid_token(auth_code.value, TokenType::Code) {
                // TODO: save token version

                debug!("received a valid authorization code, generating access and refresh tokens");

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(serde_json::to_vec(&TokenResponse::new())?))?)
            } else {
                debug!("received an invalid authorization code");

                Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("invalid auth code"))?)
            }
        }
        GrantType::RefreshToken => {
            let refresh_token: RefreshToken = de::from_bytes(&body).unwrap();

            if is_valid_token(refresh_token.value, TokenType::Refresh) {
                // TODO: increment token version

                debug!("received a valid refresh token, generating new access and refresh tokens");

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(serde_json::to_vec(&TokenResponse::new())?))?)
            } else {
                debug!("received an invalid refresh token");

                Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("invalid refresh token"))?)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct ClientCreds<'a> {
    grant_type: GrantType,
    client_id: Cow<'a, str>,
    client_secret: Cow<'a, str>,
    redirect_uri: Option<Cow<'a, str>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GrantType {
    AuthorizationCode,
    RefreshToken,
}

#[derive(Debug, Deserialize)]
struct AuthorizationCode<'a> {
    #[serde(rename = "code")]
    value: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
struct RefreshToken<'a> {
    #[serde(rename = "refresh_token")]
    value: Cow<'a, str>,
}

fn validate_client_creds(client_creds: &ClientCreds) -> bool {
    let redirect_uri_valid = client_creds
        .redirect_uri
        .as_ref()
        .map(|uri| uri == "https://social.yandex.net/broker/redirect")
        .unwrap_or(true);

    client_creds.client_id == "tbd" && client_creds.client_secret == "tbd" && redirect_uri_valid
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,

    #[serde(serialize_with = "duration_ser::serialize")]
    expires_in: Duration,
}

impl TokenResponse {
    fn access_token_exp() -> Duration {
        Duration::hours(1)
    }

    fn refresh_token_exp() -> Duration {
        Duration::weeks(1)
    }

    fn new() -> TokenResponse {
        TokenResponse {
            access_token: create_token_with_expiration_in(
                Self::access_token_exp(),
                TokenType::Access,
            ),
            refresh_token: create_token_with_expiration_in(
                Self::refresh_token_exp(),
                TokenType::Refresh,
            ),
            token_type: "Bearer".to_string(),
            expires_in: Self::access_token_exp(),
        }
    }
}

mod duration_ser {
    use chrono::Duration;
    use serde::ser;

    pub fn serialize<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dur.num_seconds())
    }
}
