use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, info, warn};

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
    task, time,
};

use elisheba::{parse_token, Command as VacuumCommand, CommandResponse as VacuumCommandResponse};
use lisa::{read_from_socket, web_handler, InspiniaController, SocketHandler, StateManager};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let elisheba_token = std::env::var("ELISHEBA_TOKEN").expect("set ENV variable ELISHEBA_TOKEN");
    let elisheba_token = parse_token::<32>(&elisheba_token);

    let socket_handler = SocketHandler::new(elisheba_token);

    let (cmd_res_tx, cmd_res_rx) = mpsc::channel::<VacuumCommandResponse>(1);
    let cmd_res_rx = Arc::from(Mutex::from(cmd_res_rx));

    let state_manager = Arc::from(Mutex::from(StateManager::new()));

    let state_manager_report = state_manager.clone();
    task::spawn(async move {
        let mut timer = time::interval(Duration::from_secs(5));

        loop {
            timer.tick().await;
            let mut state_manager = state_manager_report.clone().lock_owned().await;
            state_manager.report_if_necessary().await;
        }
    });

    // TODO: move to a function
    // https://github.com/rust-lang/rust/issues/99697
    let send_vacuum_command = {
        let socket_handler = socket_handler.clone();
        let cmd_res_rx = cmd_res_rx.clone();

        move |command| {
            debug!("wants to send command {:?}", command);

            let socket_handler = socket_handler.clone();
            let cmd_res_rx = cmd_res_rx.clone();

            async move {
                let mut socket_handler = socket_handler.clone();
                let mut cmd_res_rx = cmd_res_rx.clone().lock_owned().await;

                debug!("sending command {:?}", command);

                socket_handler.send_command(&command).await?;

                info!("sent command {:?}", command);

                let response = time::timeout(Duration::from_secs(5), cmd_res_rx.recv()).await?;
                SocketHandler::handle_response(response)
            }
        }
    };

    let send_vacuum_command = Arc::from(Mutex::from(send_vacuum_command));

    let token = std::env::var("INSPINIA_TOKEN").expect("set ENV variable INSPINIA_TOKEN");
    let inspinia_controller = InspiniaController::new(token, state_manager.clone()).await?;

    let (server, tcp, ws) = tokio::try_join!(
        task::spawn(listen_web(
            send_vacuum_command,
            inspinia_controller.clone(),
            state_manager.clone(),
        )),
        task::spawn(listen_socket(
            socket_handler,
            cmd_res_tx,
            state_manager.clone()
        )),
        task::spawn(listen_web_socket(inspinia_controller)),
    )?;

    server?;
    tcp?;
    ws?;

    Ok(())
}

async fn listen_web<F>(
    send_vacuum_command: Arc<Mutex<impl Fn(VacuumCommand) -> F + Send + Sync + 'static>>,
    inspinia_controller: InspiniaController,
    state_manager: Arc<Mutex<StateManager>>,
) -> Result<()>
where
    F: std::future::Future<Output = Result<()>> + Send + Sync + 'static,
{
    let make_svc = make_service_fn(move |_| {
        let send_vacuum_command = send_vacuum_command.clone();
        let state_manager = state_manager.clone();
        let inspinia_controller = inspinia_controller.clone();

        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let send_vacuum_command = send_vacuum_command.clone();
                let state_manager = state_manager.clone();
                let inspinia_controller = inspinia_controller.clone();

                async move {
                    web_handler(req, send_vacuum_command, inspinia_controller, state_manager).await
                }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening http://{}", addr);
    server.await?;

    Ok(())
}

async fn listen_socket(
    socket_handler: SocketHandler,
    cmd_res_tx: mpsc::Sender<VacuumCommandResponse>,
    state_manager: Arc<Mutex<StateManager>>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    let tcp_listener = TcpListener::bind(addr).await?;

    info!("Listening socket {}", addr);

    loop {
        match tcp_listener.accept().await {
            Ok((stream, addr)) => {
                let mut socket_handler = socket_handler.clone();
                socket_handler.set_stream(stream).await;

                let cmd_res_tx = cmd_res_tx.clone();
                let state_manager = state_manager.clone();

                let _ = read_from_socket(socket_handler, addr, cmd_res_tx, state_manager).await;
            }
            Err(error) => eprintln!("{}", error),
        }
    }
}

async fn listen_web_socket(mut controller: InspiniaController) -> Result<()> {
    loop {
        controller.listen().await?;

        info!("reconnecting to Inspinia...");

        let mut attempt = 1;
        while let Err(error) = controller.reconnect().await {
            warn!("failed to reconnect to Inspinia: {}", error);

            let delay = delay_for_attempt(attempt);
            warn!("timeout {} sec", delay);
            time::sleep(Duration::from_secs(delay)).await;

            attempt += 1;
        }

        info!("reconnected to Inspinia!");
    }
}

fn delay_for_attempt(attempt: u8) -> u64 {
    let delay = (attempt as f64) * 0.5 + 1_f64;
    let delay = delay.exp() * 100_f64;
    10.min(delay as u64)
}
