use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;

use hex_literal::hex;
use log::{error, info};
use tokio::io::{AsyncWriteExt, BufWriter, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use elisheva::{Command, CommandResponse, SensorData};
use isabel::{FanSpeed, Result, SocketHandler, Vacuum};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let mut vacuum = Vacuum::new([10, 0, 1, 150], hex!("59565144447659713237774434425a7a"));
    let status = vacuum.status().await?;

    let vacuum = Arc::from(Mutex::from(vacuum));

    info!("battery {}", status.battery);
    info!("bin_type {}", status.bin_type);
    info!("state {}", status.state);
    info!("fan_speed {}", status.fan_speed);

    let mut addrs = "lisa.burdukov.by:8081".to_socket_addrs()?;
    let addr = addrs.next().unwrap();

    let mut connection = SocketHandler::new();

    loop {
        let vacuum = Arc::clone(&vacuum);

        info!("connecting to {}", addr);

        match TcpStream::connect(addr).await {
            Ok(stream) => {
                info!("connected to {}", addr);

                connection.set_stream(stream);

                match connection
                    .read_commands(|command| {
                        let vacuum = Arc::clone(&vacuum);
                        handle_command(command, vacuum)
                    })
                    .await
                {
                    Ok(_) => (),
                    Err(err) => error!("{}", err),
                }

                info!("disconnected from {}", addr);
            }
            Err(_) => {
                error!("unable to connect to {}", addr);
                tokio::time::sleep(Duration::from_secs(20)).await
            }
        }
    }
}

async fn handle_command(command: Command, vacuum: Arc<Mutex<Vacuum>>) -> CommandResponse {
    let (command, result) = match command {
        Command::Start { rooms } => {
            info!("wants to start cleaning in rooms: {:?}", rooms);

            let vacuum = Arc::clone(&vacuum);
            let mut vacuum = vacuum.lock_owned().await;
            ("start", vacuum.start(rooms).await)
        }
        Command::Stop => {
            info!("wants to stop cleaning");

            let vacuum = Arc::clone(&vacuum);
            let mut vacuum = vacuum.lock_owned().await;

            ("stop", vacuum.stop().await)
        }
        Command::GoHome => {
            info!("wants to go home");

            let vacuum = Arc::clone(&vacuum);
            let mut vacuum = vacuum.lock_owned().await;

            ("go home", vacuum.go_home().await)
        }
        Command::SetWorkSpeed { mode } => {
            info!("wants to set mode {}", mode);

            let vacuum = Arc::clone(&vacuum);
            let mut vacuum = vacuum.lock_owned().await;

            let mode = FanSpeed::from(mode);
            ("set mode", vacuum.set_fan_speed(mode).await)
        }
    };

    match result {
        Ok(_) => {
            info!("ok {}", command);
            CommandResponse::Ok
        }
        Err(err) => {
            error!("err {} {}", command, err);
            CommandResponse::Failure
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
