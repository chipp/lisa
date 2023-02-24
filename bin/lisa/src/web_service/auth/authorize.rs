use std::borrow::Cow;

use bytes::Buf;
use chrono::Duration;
use hyper::{header, Body, Request, Response, StatusCode};
use log::info;
use serde::Deserialize;
use serde_urlencoded::de;
use url::Url;

use crate::Result;

use super::auth_page::AuthParams;
use super::token::{create_token_with_expiration_in, TokenType};

pub async fn authorize(request: Request<Body>) -> Result<Response<Body>> {
    let body = hyper::body::aggregate(request).await?;

    let credentials = de::from_bytes(body.chunk()).unwrap();

    if verify_credentials(credentials) {
        let auth_params = de::from_bytes(body.chunk()).unwrap();
        let redirect_url = get_redirect_url_from_params(auth_params).unwrap();

        info!("received credentials, generating an authorization code");

        Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, redirect_url.as_str())
            .body(Body::empty())?)
    } else {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("invalid request"))?;

        Ok(response)
    }
}

fn get_redirect_url_from_params(auth: AuthParams) -> Option<Url> {
    let mut url = Url::parse(auth.redirect_uri.as_ref()).ok()?;

    let code = create_token_with_expiration_in(Duration::seconds(30), TokenType::Code);
    url.query_pairs_mut()
        .append_pair("state", &auth.state)
        .append_pair("code", &code);

    Some(url)
}

#[derive(Debug, Deserialize)]
struct Credentials<'a> {
    user: Cow<'a, str>,
    password: Cow<'a, str>,
}

fn verify_credentials(credentials: Credentials) -> bool {
    let user = std::env::var("LISA_USER").expect("Set LISA_USER env variable");
    let password = std::env::var("LISA_PASSWORD").expect("Set LISA_USER env variable");

    credentials.user == user && credentials.password == password
}
