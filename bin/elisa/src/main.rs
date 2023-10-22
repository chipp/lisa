use elisa::{set_topics_and_qos, Capability, Result, Room, Topic};
use xiaomi::FanSpeed;
use xiaomi::{parse_token, Vacuum};

use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{error, info};
use paho_mqtt as mqtt;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let vacuum_token = std::env::var("VACUUM_TOKEN").expect("set ENV variable VACUUM_TOKEN");
    let vacuum_token = parse_token::<16>(&vacuum_token);

    let vacuum_ip = std::env::var("VACUUM_IP")
        .unwrap_or("10.0.1.150".to_string())
        .parse()?;

    let vacuum = Arc::from(Mutex::from(Vacuum::new(vacuum_ip, vacuum_token)));

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_client = connect_mqtt(mqtt_address).await?;
    info!("connected mqtt");

    let set_handle = task::spawn(subscribe_set(mqtt_client.clone(), vacuum.clone())).await;

    // let (set_handle, state_handle) = tokio::try_join!(
    //     task::spawn(subscribe_set(mqtt_client.clone(), vacuum.clone())),
    //     task::spawn(subscribe_state(mqtt_client, vacuum))
    // )?;

    let _ = set_handle?;
    // state_handle?;

    Ok(())
}

async fn connect_mqtt(address: String) -> Result<mqtt::AsyncClient> {
    let client = mqtt::AsyncClient::new(address).unwrap_or_else(|err| {
        error!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new_v3()
        .keep_alive_interval(Duration::from_secs(30))
        .clean_session(false)
        .finalize();

    client.connect(conn_opts).await?;

    Ok(client)
}

async fn subscribe_set(mut mqtt: mqtt::AsyncClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let (topics, qos) = set_topics_and_qos();
    mqtt.subscribe_many(&topics, &qos);

    info!("Subscribed to topics: {:?}", topics);

    while let Some(msg_opt) = stream.next().await {
        let vacuum = &mut vacuum.lock().await;

        if let Some(msg) = msg_opt {
            match Topic::from_str(msg.topic()) {
                Ok(topic) => match update_state(topic, msg.payload(), vacuum).await {
                    Ok(_) => (),
                    Err(err) => error!("Error updating state: {}", err),
                },
                Err(err) => error!("unable to parse topic {} {}", msg.topic(), err),
            }
        } else {
            error!("Lost MQTT connection. Attempting reconnect.");
            while let Err(err) = mqtt.reconnect().await {
                error!("Error MQTT reconnecting: {}", err);
                time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    Ok(())
}

async fn update_state(topic: Topic, payload: &[u8], vacuum: &mut Vacuum) -> Result<()> {
    use serde::de::{value, Error};

    let value: serde_json::Value = serde_json::from_slice(payload)?;

    match topic.capability {
        Capability::Start => {
            let rooms: Vec<Room> = serde_json::from_value(value)?;
            let room_ids = rooms.iter().map(|room| room.vacuum_id()).collect();

            info!("wants to start cleaning in rooms: {:?}", rooms);
            vacuum.start(room_ids).await
        }
        Capability::Stop => {
            info!("wants to stop cleaning");
            vacuum.stop().await?;
            vacuum.go_home().await
        }
        Capability::FanSpeed => {
            let mode = value
                .as_str()
                .ok_or(value::Error::custom("expected FanSpeed as string"))?;
            let mode = FanSpeed::from_str(mode)?;

            info!("wants to set mode {}", mode);
            vacuum.set_fan_speed(mode).await
        }
        Capability::Pause => {
            info!("wants to pause");
            vacuum.pause().await
        }
        Capability::Resume => {
            info!("wants to resume");
            vacuum.resume().await
        }
    }
}
