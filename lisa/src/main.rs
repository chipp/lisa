use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use alice::service;
use lisa::handler;

type ErasedError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), ErasedError> {
    pretty_env_logger::init();

    let make_svc = make_service_fn(|_| async {
        Ok::<_, ErasedError>(service_fn(move |req| service(req, handler)))
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening http://{}", addr);

    server.await?;

    Ok(())
}
