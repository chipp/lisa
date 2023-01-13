use std::net::ToSocketAddrs;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use log::{debug, error, info};
use tokio::{net::TcpStream, task::JoinHandle};
use tokio::{task, time::timeout};

use alzhbeta::{CommonScanner, Event, MacAddr, Scanner};
use elisheba::{parse_token, SensorData, SensorRoom};
use isabel::{Result, SocketHandler, Vacuum, VacuumController};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let vacuum_token = std::env::var("VACUUM_TOKEN").expect("set ENV variable VACUUM_TOKEN");
    let vacuum_token = parse_token::<16>(&vacuum_token);

    let vacuum_ip = std::env::var("VACUUM_IP")
        .unwrap_or("10.0.1.150".to_string())
        .parse()?;

    let mut vacuum = Vacuum::new(vacuum_ip, vacuum_token);
    let status = vacuum.status().await?;

    let vacuum_controller = VacuumController::new(vacuum);

    info!("battery {}", status.battery);
    info!("bin_type {}", status.bin_type);
    info!("state {}", status.state);
    info!("fan_speed {}", status.fan_speed);

    let server_addr = std::env::var("LISA_SOCKET_ADDR").unwrap_or("127.0.0.1:8081".to_string());

    let elisheba_token = std::env::var("ELISHEBA_TOKEN").expect("set ENV variable ELISHEBA_TOKEN");
    let elisheba_token = parse_token::<32>(&elisheba_token);

    let mut addrs = server_addr.to_socket_addrs()?;
    let addr = addrs.next().unwrap();

    let mut socket_handler = SocketHandler::new(elisheba_token);

    loop {
        debug!("connecting to {}", addr);

        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let vacuum_controller = vacuum_controller.clone();

                info!("connected to {}", addr);

                socket_handler.set_stream(stream).await;

                let abort = Arc::from(AtomicBool::from(false));

                report_vacuum_status(
                    vacuum_controller.clone(),
                    socket_handler.clone(),
                    abort.clone(),
                );

                report_sensors_task(socket_handler.clone(), abort.clone());

                match socket_handler
                    .read_commands(|command| vacuum_controller.handle_vacuum_command(command))
                    .await
                {
                    Ok(_) => (),
                    Err(err) => error!("read commands error: {}", err),
                }

                abort.store(true, Ordering::Relaxed);

                info!("disconnected from {}", addr);
            }
            Err(_) => {
                error!("unable to connect to {}", addr);
                tokio::time::sleep(Duration::from_secs(10)).await
            }
        }
    }
}

fn report_vacuum_status(
    vacuum_controller: VacuumController,
    socket_handler: SocketHandler,
    abort: Arc<AtomicBool>,
) -> JoinHandle<()> {
    task::spawn(async move {
        let mut rx = vacuum_controller.observe_vacuum_status();
        let mut socket_handler = socket_handler;

        loop {
            if abort.load(Ordering::Relaxed) {
                debug!("aborted vacuum status observing");
                break;
            }

            let status = match timeout(Duration::from_secs(5), rx.recv()).await {
                Ok(Some(status)) => status,
                Ok(None) => break,
                Err(_) => continue,
            };

            debug!("sending vacuum status {:?}", status);

            if let Err(error) = socket_handler.report_vacuum_status(status).await {
                error!("unable to send vacuum status {}", error);
            }
        }
    })
}

fn report_sensors_task(socket_handler: SocketHandler, abort: Arc<AtomicBool>) -> JoinHandle<()> {
    let mut scanner = Scanner::new();

    fn match_addr_to_room(addr: MacAddr) -> Option<SensorRoom> {
        match addr.octets {
            [0x58, 0x2d, 0x34, 0x39, 0x95, 0xf2] => Some(SensorRoom::Bedroom),
            [0x4c, 0x65, 0xa8, 0xdd, 0x82, 0xcf] => Some(SensorRoom::HomeOffice),
            [0x58, 0x2d, 0x34, 0x39, 0x97, 0x66] => Some(SensorRoom::Kitchen),
            [0x58, 0x2d, 0x34, 0x36, 0x32, 0x9b] => Some(SensorRoom::Nursery),
            _ => None,
        }
    }

    debug!("start listening bluetooth");

    task::spawn(async move {
        let mut socket_handler = socket_handler;
        let mut rx = scanner.start_scan();

        loop {
            if abort.load(Ordering::Relaxed) {
                debug!("aborted sensors observing");
                break;
            }

            let (addr, event) = match timeout(Duration::from_secs(5), rx.recv()).await {
                Ok(Some(tuple)) => tuple,
                Ok(None) => break,
                Err(_) => continue,
            };

            if let Some(room) = match_addr_to_room(addr) {
                let sensor_data = match event {
                    Event::Temperature(temperature) => {
                        SensorData::Temperature { room, temperature }
                    }
                    Event::Humidity(humidity) => SensorData::Humidity { room, humidity },
                    Event::Battery(battery) => SensorData::Battery { room, battery },
                    Event::TemperatureAndHumidity(temperature, humidity) => {
                        SensorData::TemperatureAndHumidity {
                            room,
                            temperature,
                            humidity,
                        }
                    }
                };

                debug!("sending sensor data {:?}", sensor_data);

                if let Err(error) = socket_handler.report_sensor_data(sensor_data).await {
                    error!("unable to send sensor data {}", error);
                }
            }
        }
    })
}
