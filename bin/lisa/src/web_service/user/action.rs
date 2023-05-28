use std::sync::Arc;

use alice::{UpdateStateRequest, UpdateStateResponse};
use bytes::Buf;
use elisheba::Command as VacuumCommand;
use hyper::{Body, Request, Response, StatusCode};
use tokio::sync::Mutex;

use super::super::auth::validate_autorization;
use crate::{update_devices_state, InspiniaController, Result};

pub async fn action<F>(
    request: Request<Body>,
    send_vacuum_command: Arc<Mutex<impl Fn(VacuumCommand) -> F>>,
    inspinia_controller: Arc<Mutex<InspiniaController>>,
) -> Result<Response<Body>>
where
    F: std::future::Future<Output = Result<()>>,
{
    validate_autorization(request, "devices_action", |request| async move {
        let request_id = String::from(std::str::from_utf8(
            request.headers().get("X-Request-Id").unwrap().as_bytes(),
        )?);

        let body = hyper::body::aggregate(request).await?;
        unsafe {
            println!("[action]: {}", std::str::from_utf8_unchecked(body.chunk()));
        }

        let action: UpdateStateRequest = serde_json::from_slice(body.chunk())?;
        let devices = update_devices_state(
            action.payload.devices,
            send_vacuum_command,
            inspinia_controller,
        )
        .await;

        let response = UpdateStateResponse::new(request_id, devices);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}
