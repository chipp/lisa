use std::borrow::Cow;

use axum::{http::StatusCode, response::IntoResponse, Form, Json};
use chrono::Duration;
use log::{debug, error};
use serde::{Deserialize, Serialize};

use super::token::{create_token_with_expiration_in, is_valid_token, TokenError, TokenType};

pub async fn issue_token(Form(client_creds): Form<Creds<'_>>) -> impl IntoResponse {
    if !validate_client_creds(&client_creds) {
        return (
            StatusCode::BAD_REQUEST,
            Json(Response::failure("invalid client creds".to_string())),
        );
    }

    match client_creds {
        Creds::AuthorizationCode { value, .. } => {
            if is_valid_token(value, TokenType::Code) {
                // TODO: save token version

                debug!("received a valid authorization code, generating access and refresh tokens");
                match Response::success() {
                    Ok(response) => (StatusCode::OK, Json(response)),
                    Err(err) => {
                        error!("failed to issue tokens from auth code: {}", err);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(Response::failure("internal server error".to_string())),
                        )
                    }
                }
            } else {
                debug!("received an invalid authorization code");

                (
                    StatusCode::BAD_REQUEST,
                    Json(Response::failure("invalid auth code".to_string())),
                )
            }
        }
        Creds::RefreshToken { value, .. } => {
            if is_valid_token(value, TokenType::Refresh) {
                // TODO: increment token version

                debug!("received a valid refresh token, generating new access and refresh tokens");

                match Response::success() {
                    Ok(response) => (StatusCode::OK, Json(response)),
                    Err(err) => {
                        error!("failed to issue tokens from refresh token: {}", err);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(Response::failure("internal server error".to_string())),
                        )
                    }
                }
            } else {
                debug!("received an invalid refresh token");

                (
                    StatusCode::BAD_REQUEST,
                    Json(Response::failure("invalid refresh token".to_string())),
                )
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "grant_type", rename_all = "snake_case")]
pub enum Creds<'a> {
    AuthorizationCode {
        #[serde(rename = "code")]
        value: Cow<'a, str>,
        client_id: Cow<'a, str>,
        client_secret: Cow<'a, str>,
        redirect_uri: Option<Cow<'a, str>>,
    },
    RefreshToken {
        #[serde(rename = "refresh_token")]
        value: Cow<'a, str>,
        client_id: Cow<'a, str>,
        client_secret: Cow<'a, str>,
        redirect_uri: Option<Cow<'a, str>>,
    },
}

impl<'a> Creds<'a> {
    pub fn redirect_uri(&self) -> Option<&Cow<'a, str>> {
        match self {
            Creds::AuthorizationCode { redirect_uri, .. } => redirect_uri.as_ref(),
            Creds::RefreshToken { redirect_uri, .. } => redirect_uri.as_ref(),
        }
    }

    pub fn client_id(&self) -> &Cow<'a, str> {
        match self {
            Creds::AuthorizationCode { client_id, .. } => client_id,
            Creds::RefreshToken { client_id, .. } => client_id,
        }
    }

    pub fn client_secret(&self) -> &Cow<'a, str> {
        match self {
            Creds::AuthorizationCode { client_secret, .. } => client_secret,
            Creds::RefreshToken { client_secret, .. } => client_secret,
        }
    }
}

fn validate_client_creds(client_creds: &Creds) -> bool {
    let redirect_uri_valid = client_creds
        .redirect_uri()
        .map(|uri| uri == "https://social.yandex.net/broker/redirect")
        .unwrap_or(true);

    client_creds.client_id() == "tbd" && client_creds.client_secret() == "tbd" && redirect_uri_valid
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Response {
    Success {
        access_token: String,
        refresh_token: String,
        token_type: String,

        #[serde(serialize_with = "duration_ser::serialize")]
        expires_in: Duration,
    },
    Failure {
        error: String,
    },
}

impl Response {
    fn success() -> Result<Response, TokenError> {
        Ok(Response::Success {
            access_token: create_token_with_expiration_in(
                ACCESS_TOKEN_EXPIRATION,
                TokenType::Access,
            )?,
            refresh_token: create_token_with_expiration_in(
                REFRESH_TOKEN_EXPIRATION,
                TokenType::Refresh,
            )?,
            token_type: "Bearer".to_string(),
            expires_in: ACCESS_TOKEN_EXPIRATION,
        })
    }

    fn failure(error: String) -> Response {
        Response::Failure { error }
    }
}

const ACCESS_TOKEN_EXPIRATION: Duration = Duration::hours(1);
const REFRESH_TOKEN_EXPIRATION: Duration = Duration::weeks(4);

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
