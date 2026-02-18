use std::borrow::Cow;

use axum::{
    http::{header::LOCATION, HeaderMap, StatusCode},
    response::IntoResponse,
    Form,
};
use chrono::Duration;
use log::{error, info};
use serde::Deserialize;
use url::Url;

use super::token::{create_token_with_expiration_in, TokenError, TokenType};

pub async fn authorize(Form(credentials): Form<Credentials<'_>>) -> impl IntoResponse {
    if verify_credentials(&credentials) {
        let redirect_url = match get_redirect_url_from_params(credentials) {
            Ok(Some(redirect_url)) => redirect_url,
            Ok(None) => return (StatusCode::BAD_REQUEST, HeaderMap::new()),
            Err(err) => {
                error!("failed to create authorization code: {}", err);
                return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new());
            }
        };

        info!("received credentials, generating an authorization code");

        let mut headers = HeaderMap::new();
        headers.append(LOCATION, redirect_url.as_str().parse().unwrap());

        (StatusCode::FOUND, headers)
    } else {
        (StatusCode::BAD_REQUEST, HeaderMap::new())
    }
}

fn get_redirect_url_from_params(auth: Credentials) -> Result<Option<Url>, TokenError> {
    let mut url = match Url::parse(auth.redirect_uri.as_ref()) {
        Ok(url) => url,
        Err(_) => return Ok(None),
    };

    let code = create_token_with_expiration_in(Duration::seconds(30), TokenType::Code)?;
    url.query_pairs_mut()
        .append_pair("state", &auth.state)
        .append_pair("code", &code);

    Ok(Some(url))
}

#[derive(Debug, Deserialize)]
pub struct Credentials<'a> {
    user: Cow<'a, str>,
    password: Cow<'a, str>,
    state: Cow<'a, str>,
    redirect_uri: Cow<'a, str>,
}

fn verify_credentials(credentials: &Credentials) -> bool {
    let user = std::env::var("LISA_USER").expect("Set LISA_USER env variable");
    let password = std::env::var("LISA_PASSWORD").expect("Set LISA_USER env variable");

    credentials.user == user && credentials.password == password
}
