use std::{net::SocketAddr, sync::Arc, time::Duration};

use elisheba::{Command, CommandResponse, Packet};
use log::info;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
    task, time,
};

use lisa::{service, SocketHandler};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let socket_handler = SocketHandler::new();
    let (tx, rx) = mpsc::channel::<CommandResponse>(1);
    let rx = Arc::from(Mutex::from(rx));

    // TODO: move to a function
    let send_command = {
        let socket_handler = socket_handler.clone();
        let rx = rx.clone();

        move |command| {
            info!("wants to send command {:?}", command);

            let socket_handler = socket_handler.clone();
            let rx = rx.clone();

            async move {
                let mut socket_handler = socket_handler.clone();
                let mut rx = rx.clone().lock_owned().await;

                info!("sending command {:?}", command);

                socket_handler.send_command(command).await?;

                let response = time::timeout(Duration::from_secs(5), rx.recv()).await?;
                SocketHandler::handle_response(response)
            }
        }
    };
    let send_command = Arc::from(Mutex::from(send_command));

    let (server, tcp) = tokio::try_join!(
        task::spawn(listen_api(send_command)),
        task::spawn(listen_tcp(socket_handler, tx))
    )?;

    server?;
    tcp?;

    Ok(())
}

async fn listen_api<F>(
    send_command: Arc<Mutex<impl Fn(Command) -> F + Send + Sync + 'static>>,
) -> Result<()>
where
    F: std::future::Future<Output = Result<()>> + Send + Sync + 'static,
{
    let make_svc = make_service_fn(move |_| {
        let send_command = send_command.clone();
        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let send_command = send_command.clone();
                async move { service(req, send_command).await }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening http://{}", addr);
    server.await?;

    Ok(())
}

async fn listen_tcp(
    socket_handler: SocketHandler,
    tx: mpsc::Sender<CommandResponse>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    let tcp_listener = TcpListener::bind(addr).await?;

    info!("Listening socket {}", addr);

    let mut handles = vec![];

    loop {
        match tcp_listener.accept().await {
            Ok((stream, addr)) => {
                let mut socket_handler = socket_handler.clone();
                socket_handler.set_stream(stream).await;

                let tx = tx.clone();

                handles.push(task::spawn(read_from_socket(socket_handler, addr, tx)))
            }
            Err(error) => eprintln!("{}", error),
        }
    }
}

async fn read_from_socket(
    socket_handler: SocketHandler,
    addr: SocketAddr,
    tx: mpsc::Sender<CommandResponse>,
) -> Result<()> {
    info!("Got a new client {}", addr);

    let mut socket_handler = socket_handler;
    socket_handler
        .read_packets(|packet| {
            let tx = tx.clone();

            async move {
                match packet {
                    Packet::CommandResponse(response) => tx.send(response).await.unwrap(),
                    Packet::VacuumBatteryPercentage(battery_percentage) => {
                        info!("battery percentage {}", battery_percentage);
                    }
                }
            }
        })
        .await?;

    info!("Client did disconnect {}", addr);

    Ok(())
}
