use std::time::Duration;
use std::{net::ToSocketAddrs, sync::Arc};

use hex_literal::hex;
use log::{debug, error, info};
use tokio::task;
use tokio::{net::TcpStream, sync::Mutex};

use elisheba::SensorData;
use isabel::{Result, SocketHandler, Vacuum, VacuumController};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let (tx, rx) = std::sync::mpsc::sync_channel::<u8>(1);
    let rx = Arc::from(Mutex::from(rx));

    std::thread::spawn(move || loop {
        tx.send(25).unwrap() // stub data
    });

    let mut vacuum = Vacuum::new([10, 0, 1, 150], hex!("59565144447659713237774434425a7a"));
    let status = vacuum.status().await?;

    let vacuum_controller = VacuumController::new(vacuum);

    info!("battery {}", status.battery);
    info!("bin_type {}", status.bin_type);
    info!("state {}", status.state);
    info!("fan_speed {}", status.fan_speed);

    let server_addr = std::env::var("LISA_SOCKET_ADDR").unwrap_or("localhost:8081".to_string());

    let mut addrs = server_addr.to_socket_addrs()?;
    let addr = addrs.next().unwrap();

    let mut socket_handler = SocketHandler::new();

    loop {
        debug!("connecting to {}", addr);

        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let vacuum_controller = vacuum_controller.clone();
                let rx = rx.clone();

                info!("connected to {}", addr);

                socket_handler.set_stream(stream);

                let report_battery_task =
                    report_battery_task(vacuum_controller.clone(), socket_handler.clone());

                let report_sensors_task = report_sensors_task(socket_handler.clone(), rx);

                match socket_handler
                    .read_commands(|command| vacuum_controller.handle_vacuum_command(command))
                    .await
                {
                    Ok(_) => (),
                    Err(err) => error!("{}", err),
                }

                report_battery_task.abort();
                report_sensors_task.abort();

                info!("disconnected from {}", addr);
            }
            Err(_) => {
                error!("unable to connect to {}", addr);
                tokio::time::sleep(Duration::from_secs(20)).await
            }
        }
    }
}

fn report_battery_task(
    vacuum_controller: VacuumController,
    socket_handler: SocketHandler,
) -> tokio::task::JoinHandle<()> {
    task::spawn(async move {
        let mut rx = vacuum_controller.observe_vacuum_status();
        let mut socket_handler = socket_handler;

        while let Some(status) = rx.recv().await {
            debug!("sending vacuum status {:?}", status);

            if let Err(error) = socket_handler.report_vacuum_status(status).await {
                error!("unable to send vacuum status {}", error);
            }
        }
    })
}

fn report_sensors_task(
    socket_handler: SocketHandler,
    rx: Arc<Mutex<std::sync::mpsc::Receiver<u8>>>,
) -> tokio::task::JoinHandle<()> {
    task::spawn(async move {
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(120));
        let mut socket_handler = socket_handler;

        loop {
            timer.tick().await;

            let rx = rx.clone().lock_owned().await;

            let temp = rx.recv().unwrap() + 200;
            let temp = temp as f32 / 10.0;

            let sensor_data = SensorData {
                room: "nursery".to_owned(),
                temperature: temp,
                humidity: 52.0,
                battery: 100,
            };

            debug!("sending sensor data {:?}", sensor_data);

            if let Err(error) = socket_handler.report_sensor_data(sensor_data).await {
                error!("unable to send sensor data {}", error);
            }
        }
    })
}
