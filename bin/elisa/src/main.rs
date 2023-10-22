use elisa::{actions_topics_and_qos, room_id_for_room, topic_for_state};
use elisa::{Action, Result, Room, State, Status};
use topics::Topic;
use xiaomi::{parse_token, FanSpeed, Vacuum};

use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{error, info};
use paho_mqtt as mqtt;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{self, interval};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let vacuum_token = std::env::var("VACUUM_TOKEN").expect("set ENV variable VACUUM_TOKEN");
    let vacuum_token = parse_token::<16>(&vacuum_token);

    let vacuum_ip = std::env::var("VACUUM_IP")
        .unwrap_or("10.0.1.150".to_string())
        .parse()?;

    let mut vacuum = Vacuum::new(vacuum_ip, vacuum_token);
    if let Ok(status) = vacuum.status().await {
        info!("vacuum status: {:?}", status);
    }

    let vacuum = Arc::from(Mutex::from(vacuum));

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_client = connect_mqtt(mqtt_address).await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_actions(mqtt_client.clone(), vacuum.clone())),
        task::spawn(subscribe_state(mqtt_client, vacuum))
    )?;

    set_handle?;
    state_handle?;

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

async fn subscribe_actions(mut mqtt: mqtt::AsyncClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let (topics, qos) = actions_topics_and_qos();
    mqtt.subscribe_many(&topics, &qos);

    info!("Subscribed to topics: {:?}", topics);

    while let Some(msg_opt) = stream.next().await {
        let vacuum = &mut vacuum.lock().await;

        if let Some(msg) = msg_opt {
            match Topic::from_str(msg.topic()) {
                Ok(topic) => match perform_action(topic, msg.payload(), vacuum).await {
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

async fn perform_action(topic: Topic<Action>, payload: &[u8], vacuum: &mut Vacuum) -> Result<()> {
    use serde::de::{value, Error};

    let value: serde_json::Value = serde_json::from_slice(payload)?;

    match topic.feature {
        Action::Start => {
            let rooms: Vec<Room> = serde_json::from_value(value)?;
            let room_ids = rooms.iter().map(room_id_for_room).collect();

            info!("wants to start cleaning in rooms: {:?}", rooms);
            vacuum.start(room_ids).await
        }
        Action::Stop => {
            info!("wants to stop cleaning");
            vacuum.stop().await?;
            vacuum.go_home().await
        }
        Action::SetFanSpeed => {
            let mode = value
                .as_str()
                .ok_or(value::Error::custom("expected FanSpeed as string"))?;
            let mode = FanSpeed::from_str(mode)?;

            info!("wants to set mode {}", mode);
            vacuum.set_fan_speed(mode).await
        }
        Action::Pause => {
            info!("wants to pause");
            vacuum.pause().await
        }
        Action::Resume => {
            info!("wants to resume");
            vacuum.resume().await
        }
    }
}

async fn subscribe_state(mqtt: mqtt::AsyncClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut timer = interval(Duration::from_secs(10));

    loop {
        timer.tick().await;
        let mut vacuum = vacuum.lock().await;

        if let Ok(status) = vacuum.status().await {
            info!("publishing state: {:?}", status);

            let status = Status::from(status);
            let topic = topic_for_state(State::Status);
            let payload = serde_json::to_vec(&status).unwrap();

            let message = mqtt::MessageBuilder::new()
                .topic(topic.to_string())
                .payload(payload)
                .finalize();

            mqtt.publish(message).await?;
        }
    }
}
