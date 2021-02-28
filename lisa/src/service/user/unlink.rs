use hyper::{Body, Request, Response, StatusCode};

use super::super::auth::validate_autorization;
use crate::Result;

pub async fn unlink(request: Request<Body>) -> Result<Response<Body>> {
    validate_autorization(request, "devices_action", |_| async move {
        Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("invalid request"))?)
    })
    .await
}
