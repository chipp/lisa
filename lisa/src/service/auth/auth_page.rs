use std::borrow::Cow;

use serde::Deserialize;

use hyper::{Body, Request, Response, StatusCode};
use log::info;

use crate::Result;

pub fn auth_page(request: Request<Body>) -> Result<Response<Body>> {
    Ok(match params_for_auth_page(&request).and_then(auth_html) {
        Some(html) => {
            info!("starting authentication process");

            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(html))?
        }
        None => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("invalid request"))?,
    })
}

#[derive(Debug, Deserialize)]
pub struct AuthParams<'a> {
    pub state: Cow<'a, str>,
    pub redirect_uri: Cow<'a, str>,
    pub response_type: Cow<'a, str>,
    pub client_id: Cow<'a, str>,
}

static AUTH_HTML: &str = include_str!("./auth_page.html");

fn params_for_auth_page<'a>(request: &'a Request<Body>) -> Option<AuthParams> {
    let query = request.uri().query()?;
    serde_urlencoded::de::from_str(query).ok()
}

fn auth_html(auth: AuthParams) -> Option<String> {
    let mut html = String::from(AUTH_HTML);

    html = html.replace("#CLIENT_ID#", auth.client_id.as_ref());
    html = html.replace("#RESPONSE_TYPE#", auth.response_type.as_ref());
    html = html.replace("#REDIRECT_URI#", auth.redirect_uri.as_ref());
    html = html.replace("#STATE#", auth.state.as_ref());

    Some(html)
}
