use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use log::{debug, info};

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
    task, time,
};

use elisheba::{parse_token, Command, CommandResponse, Packet, SensorData, SensorRoom};
use lisa::{service, SocketHandler, StateManager};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let elisheba_token = std::env::var("ELISHEBA_TOKEN").expect("set ENV variable ELISHEBA_TOKEN");
    let elisheba_token = parse_token::<32>(&elisheba_token);

    let socket_handler = SocketHandler::new(elisheba_token);

    let (cmd_res_tx, cmd_res_rx) = mpsc::channel::<CommandResponse>(1);
    let cmd_res_rx = Arc::from(Mutex::from(cmd_res_rx));

    let state_manager = Arc::from(Mutex::from(StateManager::new()));

    // TODO: move to a function
    let send_command = {
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
    let send_command = Arc::from(Mutex::from(send_command));

    let (server, tcp) = tokio::try_join!(
        task::spawn(listen_http(send_command, state_manager.clone())),
        task::spawn(listen_tcp(socket_handler, cmd_res_tx, state_manager))
    )?;

    server?;
    tcp?;

    Ok(())
}

async fn listen_http<F>(
    send_command: Arc<Mutex<impl Fn(Command) -> F + Send + Sync + 'static>>,
    state_manager: Arc<Mutex<StateManager>>,
) -> Result<()>
where
    F: std::future::Future<Output = Result<()>> + Send + Sync + 'static,
{
    let make_svc = make_service_fn(move |_| {
        let send_command = send_command.clone();
        let state_manager = state_manager.clone();

        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let send_command = send_command.clone();
                let state_manager = state_manager.clone();

                async move { service(req, send_command, state_manager).await }
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
    cmd_res_tx: mpsc::Sender<CommandResponse>,
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

async fn read_from_socket(
    socket_handler: SocketHandler,
    addr: SocketAddr,
    cmd_res_tx: mpsc::Sender<CommandResponse>,
    state_manager: Arc<Mutex<StateManager>>,
) -> Result<()> {
    info!("A client did connect {}", addr);

    let abort = Arc::from(AtomicBool::from(false));

    let state_manager_report = state_manager.clone();
    let abort_report = abort.clone();
    task::spawn(async move {
        let mut timer = time::interval(Duration::from_secs(30));

        loop {
            if abort_report.load(Ordering::Relaxed) {
                break;
            }

            timer.tick().await;
            let mut state_manager = state_manager_report.clone().lock_owned().await;
            state_manager.report_if_necessary().await;
        }
    });

    let mut socket_handler = socket_handler;
    let _ = socket_handler
        .read_packets(|packet| {
            let cmd_res_tx = cmd_res_tx.clone();
            let state_manager = state_manager.clone();

            async move {
                match packet {
                    Packet::CommandResponse(response) => cmd_res_tx.send(response).await.unwrap(),
                    Packet::VacuumStatus(status) => {
                        let mut state = state_manager.clone().lock_owned().await;

                        state.vacuum_state.set_battery(status.battery);
                        state.vacuum_state.set_is_enabled(status.is_enabled);
                        state.vacuum_state.set_work_speed(status.work_speed);
                    }
                    Packet::SensorData(sensor_data) => {
                        let mut state = state_manager.clone().lock_owned().await;

                        let room_state = match sensor_data.room() {
                            SensorRoom::Bedroom => &mut state.bedroom_sensor_state,
                            SensorRoom::HomeOffice => &mut state.home_office_sensor_state,
                            SensorRoom::Kitchen => &mut state.kitchen_sensor_state,
                        };

                        match sensor_data {
                            SensorData::Temperature {
                                room: _,
                                temperature,
                            } => {
                                room_state.set_temperature(temperature);
                            }
                            SensorData::Humidity { room: _, humidity } => {
                                room_state.set_humidity(humidity);
                            }
                            SensorData::Battery { room: _, battery } => {
                                room_state.set_battery(battery);
                            }
                            SensorData::TemperatureAndHumidity {
                                room: _,
                                temperature,
                                humidity,
                            } => {
                                room_state.set_temperature(temperature);
                                room_state.set_humidity(humidity);
                            }
                        }
                    }
                }
            }
        })
        .await;

    abort.store(true, Ordering::Relaxed);

    info!("The client did disconnect {}", addr);

    Ok(())
}
