mod auth {
    mod auth_page;
    mod authorize;
    mod issue_token;
    mod token;
    mod validate_autorization;

    pub use auth_page::auth_page;
    pub use authorize::authorize;
    pub use issue_token::issue_token;
    pub use validate_autorization::validate_autorization;
}

mod user {
    mod action;
    mod devices;
    mod pong;
    mod query;
    mod unlink;

    pub use action::action;
    pub use devices::devices;
    pub use pong::pong;
    pub use query::query;
    pub use unlink::unlink;
}

use bytes::Buf;
use log::error;

use hyper::{Body, Method, Request, Response, StatusCode};
use tokio::sync::mpsc::UnboundedSender;

use crate::{ActionPayload, Result};

pub async fn web_handler(
    request: Request<Body>,
    perform_action: UnboundedSender<ActionPayload>,
) -> Result<Response<Body>> {
    match (request.uri().path(), request.method()) {
        ("/auth", &Method::GET) => auth::auth_page(request),
        ("/auth", &Method::POST) => auth::authorize(request).await,
        ("/token", &Method::POST) => auth::issue_token(request).await,
        ("/v1.0", &Method::HEAD) => user::pong(),
        ("/v1.0/user/devices", &Method::GET) => user::devices(request).await,
        ("/v1.0/user/devices/query", &Method::POST) => user::query(request).await,
        ("/v1.0/user/devices/action", &Method::POST) => user::action(request, perform_action).await,
        ("/v1.0/user/unlink", &Method::POST) => user::unlink(request).await,
        _ => {
            error!("Unsupported request: {:?}", request);

            let body = hyper::body::aggregate(request).await?;

            match std::str::from_utf8(body.chunk()) {
                Ok(body) if !body.is_empty() => error!("Body {}", body),
                _ => (),
            }

            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("invalid request"))?;

            Ok(response)
        }
    }
}
