use bytes::Buf;
use hyper::{Body, Request, Response};
use log::trace;

use crate::web_service::auth::validate_autorization;
use crate::Result;

pub async fn query(request: Request<Body>) -> Result<Response<Body>> {
    validate_autorization(request, "devices_query", |request| async move {
        let _request_id = String::from(std::str::from_utf8(
            request.headers().get("X-Request-Id").unwrap().as_bytes(),
        )?);

        let body = hyper::body::aggregate(request).await?;
        unsafe {
            trace!("[query]: {}", std::str::from_utf8_unchecked(body.chunk()));
        }

        panic!();

        // let query: StateRequest = serde_json::from_slice(body.chunk())?;
        // let devices = {
        //     let state_manager = state_manager.lock_owned().await;
        //     query
        //         .devices
        //         .iter()
        //         .filter_map(|device| DeviceId::from_str(device.id).ok())
        //         .filter_map(|id| state_manager.state_for_device(id))
        //         .collect()
        // };

        // let response = StateResponse::new(request_id, devices);

        // Ok(Response::builder()
        //     .status(StatusCode::OK)
        //     .header("Content-Type", "application/json")
        //     .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}
