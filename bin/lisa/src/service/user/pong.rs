use hyper::{Body, Response, StatusCode};

use crate::Result;

pub fn pong() -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())?)
}
