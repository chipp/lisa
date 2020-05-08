use bytes::buf::BufExt;

use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};

use alice::{Request as AliceRequest, Response as AliceResponse};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

async fn hello(request: Request<Body>) -> Result<Response<Body>> {
    match request.method() {
        &Method::POST => {
            let body = hyper::body::aggregate(request).await?;
            let alice_request: AliceRequest = serde_json::from_reader(body.reader())?;
            let alice_response = if &alice_request.payload.original_utterance == "ping" {
                AliceResponse::from_string("pong")
            } else {
                AliceResponse::from_string("Привет! Температура в зале 23.5 градуса")
            };
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

#[tokio::main]
async fn main() -> Result<()> {

    let make_svc = make_service_fn(|_conn| async { Ok::<_, ErasedError>(service_fn(hello)) });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening http://{}", addr);

    server.await?;

    Ok(())
}
