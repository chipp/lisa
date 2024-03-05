use std::borrow::Cow;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use log::debug;
use serde::Deserialize;

pub async fn auth_page(Query(params): Query<AuthParams<'_>>) -> impl IntoResponse {
    match auth_html(params) {
        Some(html) => {
            debug!("starting authentication process");
            (StatusCode::OK, Html(html))
        }
        None => (StatusCode::BAD_REQUEST, Html("bad request".to_string())),
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthParams<'a> {
    pub state: Cow<'a, str>,
    pub redirect_uri: Cow<'a, str>,
    pub response_type: Cow<'a, str>,
    pub client_id: Cow<'a, str>,
}

static AUTH_HTML: &str = include_str!("./auth_page.html");

fn auth_html(auth: AuthParams) -> Option<String> {
    let mut html = String::from(AUTH_HTML);

    html = html.replace("#CLIENT_ID#", auth.client_id.as_ref());
    html = html.replace("#RESPONSE_TYPE#", auth.response_type.as_ref());
    html = html.replace("#REDIRECT_URI#", auth.redirect_uri.as_ref());
    html = html.replace("#STATE#", auth.state.as_ref());

    Some(html)
}
