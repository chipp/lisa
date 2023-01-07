use std::{str::FromStr, sync::Arc};

use alice::{StateRequest, StateResponse};
use bytes::Buf;
use hyper::{Body, Request, Response, StatusCode};
use log::trace;
use tokio::sync::Mutex;

use super::super::auth::validate_autorization;
use crate::{DeviceId, Result, StateManager};

pub async fn query(
    request: Request<Body>,
    state_manager: Arc<Mutex<StateManager>>,
) -> Result<Response<Body>> {
    validate_autorization(request, "devices_query", |request| async move {
        let request_id = String::from(std::str::from_utf8(
            request.headers().get("X-Request-Id").unwrap().as_bytes(),
        )?);

        let body = hyper::body::aggregate(request).await?;
        unsafe {
            trace!("[query]: {}", std::str::from_utf8_unchecked(body.chunk()));
        }

        let query: StateRequest = serde_json::from_slice(body.chunk())?;
        let devices = {
            let state_manager = state_manager.lock_owned().await;
            query
                .devices
                .iter()
                .filter_map(|device| DeviceId::from_str(device.id).ok())
                .filter_map(|id| state_manager.state_for_device(id))
                .collect()
        };

        let response = StateResponse::new(request_id, devices);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}
