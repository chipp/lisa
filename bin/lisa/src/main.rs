use std::{net::SocketAddr, sync::Arc};

use log::info;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use tokio::{
    io::ReadHalf,
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task,
};

use lisa::{service, SocketHandler};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let socket_handler = Arc::from(Mutex::from(SocketHandler::new()));

    let (server, tcp) = tokio::try_join!(
        task::spawn(listen_api(socket_handler.clone())),
        task::spawn(listen_tcp(socket_handler))
    )?;

    server?;
    tcp?;

    Ok(())
}

async fn listen_api(cmd: Arc<Mutex<SocketHandler>>) -> Result<()> {
    let cmd = cmd.clone();

    let make_svc = make_service_fn(move |_| {
        let cmd = cmd.clone();
        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let cmd = cmd.clone();
                async move { service(req, cmd).await }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);
    
    info!("Listening http://{}", addr);

    server.await?;

    Ok(())
}

async fn listen_tcp(cmd: Arc<Mutex<SocketHandler>>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    let tcp_listener = TcpListener::bind(addr).await?;

    info!("Listening socket {}", addr);

    // let mut handles = vec![];

    loop {
        match tcp_listener.accept().await {
            Ok((stream, _addr)) => {
                {
                    let mut cmd = cmd.clone().lock_owned().await;
                    cmd.set_stream(stream);
                }

                // handles.push(task::spawn(read_from_socket(read, addr, read_cmd)))
            }
            Err(error) => eprintln!("{}", error),
        }
    }
}

async fn _read_from_socket(
    _stream: ReadHalf<TcpStream>,
    _addr: SocketAddr,
    _cmd: Arc<Mutex<SocketHandler>>,
) -> Result<()> {
    info!("Got a new client {}", _addr);

    info!("Client did disconnect {}", _addr);

    Ok(())
}
