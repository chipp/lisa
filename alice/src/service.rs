use bytes::buf::BufExt;
use hyper::{header, Body, Method, Request, Response, StatusCode};

use crate::Request as AliceRequest;
use crate::Response as AliceResponse;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn service(
    request: Request<Body>,
    handler: fn(AliceRequest) -> Result<AliceResponse>,
) -> Result<Response<Body>> {
    match request.method() {
        &Method::POST => {
            let body = hyper::body::aggregate(request).await?;

            let alice_request: AliceRequest = serde_json::from_reader(body.reader())?;
            let alice_response = handler(alice_request)?;

            let json_response = serde_json::to_vec(&alice_response)?;
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json_response))?;

            Ok(response)
        }
        _ => {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("invalid request"))?;

            Ok(response)
        }
    }
}
