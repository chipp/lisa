use std::borrow::Cow;

use axum::{
    http::{header::LOCATION, HeaderMap, StatusCode},
    response::IntoResponse,
    Form,
};
use chrono::Duration;
use log::info;
use serde::Deserialize;
use url::Url;

use super::token::{create_token_with_expiration_in, TokenType};

pub async fn authorize(Form(credentials): Form<Credentials<'_>>) -> impl IntoResponse {
    if verify_credentials(&credentials) {
        let redirect_url = get_redirect_url_from_params(credentials).unwrap();

        info!("received credentials, generating an authorization code");

        let mut headers = HeaderMap::new();
        headers.append(LOCATION, redirect_url.as_str().parse().unwrap());

        (StatusCode::FOUND, headers)
    } else {
        (StatusCode::BAD_REQUEST, HeaderMap::new())
    }
}

fn get_redirect_url_from_params(auth: Credentials) -> Option<Url> {
    let mut url = Url::parse(auth.redirect_uri.as_ref()).ok()?;

    let code = create_token_with_expiration_in(Duration::seconds(30), TokenType::Code);
    url.query_pairs_mut()
        .append_pair("state", &auth.state)
        .append_pair("code", &code);

    Some(url)
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
