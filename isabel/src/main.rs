use std::net::ToSocketAddrs;
use std::sync::Arc;

use hex_literal::hex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task;

use elisheva::{Command, SensorData};
use isabel::{FanSpeed, Result, Vacuum};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let mut vacuum = Vacuum::new([10, 0, 1, 150], hex!("59565144447659713237774434425a7a"));
    let status = vacuum.status().await?;

    let vacuum = Arc::from(Mutex::from(vacuum));

    println!("status {}", status);

    let mut addrs = "lisa.burdukov.by:8081".to_socket_addrs()?;
    let addr = addrs.next().unwrap();

    println!("connecting to {}", addr);

    let stream = TcpStream::connect(addr).await.unwrap();
    let (read, write) = tokio::io::split(stream);

    println!("connected to {}", addr);

    let read = BufReader::new(read);
    let write = BufWriter::new(write);

    let (read, write) = tokio::try_join!(
        task::spawn(timer_report_data(write)),
        task::spawn(read_remote_commands(read, vacuum))
    )?;

    read?;
    write?;

    Ok(())
}

async fn read_remote_commands(
    stream: BufReader<ReadHalf<TcpStream>>,
    vacuum: Arc<Mutex<Vacuum>>,
) -> Result<()> {
    println!("waiting for commands...");

    let mut stream = stream;

    loop {
        let mut buffer = vec![];
        stream.read_until(b'\n', &mut buffer).await?;

        match serde_json::from_slice::<Command>(&buffer) {
            Ok(Command::Start { rooms }) => {
                let vacuum = Arc::clone(&vacuum);
                let mut vacuum = vacuum.lock_owned().await;

                match vacuum.start(rooms).await {
                    Ok(_) => println!("ok start"),
                    Err(_) => eprintln!("err start"),
                }
            }
            Ok(Command::Stop) => {
                println!("got stop");

                let vacuum = Arc::clone(&vacuum);
                let mut vacuum = vacuum.lock_owned().await;

                println!("sending stop");

                match vacuum.stop().await {
                    Ok(_) => println!("ok stop"),
                    Err(_) => eprintln!("err stop"),
                }
            }
            Ok(Command::GoHome) => {
                println!("got home");

                let vacuum = Arc::clone(&vacuum);
                let mut vacuum = vacuum.lock_owned().await;

                println!("sending home");

                match vacuum.go_home().await {
                    Ok(_) => println!("ok home"),
                    Err(_) => eprintln!("err home"),
                }
            }
            Ok(Command::SetMode { mode }) => {
                let vacuum = Arc::clone(&vacuum);
                let mut vacuum = vacuum.lock_owned().await;

                let mode = FanSpeed::from(mode);
                match vacuum.set_fan_speed(mode).await {
                    Ok(_) => println!("ok set mode"),
                    Err(_) => eprintln!("err set mode"),
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}

async fn timer_report_data(stream: BufWriter<WriteHalf<TcpStream>>) -> Result<()> {
    let mut stream = stream;
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        timer.tick().await;
        report_data(&mut stream).await?;
    }
}

async fn report_data(stream: &mut BufWriter<WriteHalf<TcpStream>>) -> Result<()> {
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
