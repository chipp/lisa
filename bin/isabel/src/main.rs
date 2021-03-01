use std::net::ToSocketAddrs;
use std::time::Duration;

use hex_literal::hex;
use log::{debug, error, info};
use tokio::net::TcpStream;
use tokio::{
    io::{AsyncWriteExt, BufWriter, WriteHalf},
    task,
};

use elisheba::SensorData;
use isabel::{Result, SocketHandler, Vacuum, VacuumController};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let mut vacuum = Vacuum::new([10, 0, 1, 150], hex!("59565144447659713237774434425a7a"));
    let status = vacuum.status().await?;

    let vacuum_controller = VacuumController::new(vacuum);

    info!("battery {}", status.battery);
    info!("bin_type {}", status.bin_type);
    info!("state {}", status.state);
    info!("fan_speed {}", status.fan_speed);

    let mut addrs = "lisa.burdukov.by:8081".to_socket_addrs()?;
    let addr = addrs.next().unwrap();

    let mut connection = SocketHandler::new();

    loop {
        debug!("connecting to {}", addr);

        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let vacuum_controller = vacuum_controller.clone();

                info!("connected to {}", addr);

                connection.set_stream(stream);

                let report_battery_task = {
                    let vacuum_controller = vacuum_controller.clone();
                    let connection = connection.clone();

                    task::spawn(async move {
                        let mut rx = vacuum_controller.observe_vacuum_status();
                        let mut connection = connection;

                        while let Some(status) = rx.recv().await {
                            debug!("sending battery percentage {}", status.battery);

                            if let Err(error) = connection.report_vacuum_status(status).await {
                                error!("unable to send battery percentage {}", error);
                            }
                        }
                    })
                };

                match connection
                    .read_commands(|command| vacuum_controller.handle_vacuum_command(command))
                    .await
                {
                    Ok(_) => (),
                    Err(err) => error!("{}", err),
                }

                report_battery_task.abort();
                info!("disconnected from {}", addr);
            }
            Err(_) => {
                error!("unable to connect to {}", addr);
                tokio::time::sleep(Duration::from_secs(20)).await
            }
        }
    }
}

async fn _timer_report_data(stream: BufWriter<WriteHalf<TcpStream>>) -> Result<()> {
    let mut stream = stream;
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        timer.tick().await;
        _report_data(&mut stream).await?;
    }
}

async fn _report_data(stream: &mut BufWriter<WriteHalf<TcpStream>>) -> Result<()> {
    let data = SensorData {
        temperature: 25.5,
        humidity: 52.0,
        battery: 100,
    };

    let bytes = serde_json::to_vec(&data)?;

    stream.write_all(&bytes).await?;
    stream.write_all(b"\n").await?;

    stream.flush().await?;

    Ok(())
}
