use std::net::ToSocketAddrs;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::task;

use elisheva::{Command, SensorData};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
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
        task::spawn(read_remote_commands(read))
    )?;

    read?;
    write?;

    Ok(())
}

async fn read_remote_commands(stream: BufReader<ReadHalf<TcpStream>>) -> Result<()> {
    println!("waiting for commands...");

    let mut stream = stream;
    loop {
        let mut buffer = vec![];
        stream.read_until(b'\n', &mut buffer).await?;

        match serde_json::from_slice::<Command>(&buffer) {
            Ok(Command::Start { rooms }) => println!("start cleaning {:?}", rooms),
            Ok(Command::SetMode { mode }) => println!("set mode {}", mode),
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
