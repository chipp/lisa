use elisa::{perform_action, prepare_state, Result, Storage};
use transport::Topic;
use xiaomi::{parse_token, Vacuum};

use std::process;
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

    mqtt.subscribe_many(&[Topic::elisa_action().to_string()], &[mqtt::QOS_1]);

    info!("Subscribed to topic: {}", Topic::elisa_action());

    while let Some(msg_opt) = stream.next().await {
        let vacuum = &mut vacuum.lock().await;

        if let Some(msg) = msg_opt {
            match perform_action(msg.payload(), vacuum).await {
                Ok(_) => (),
                Err(err) => error!("Error updating state: {}", err),
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

async fn subscribe_state(mqtt: mqtt::AsyncClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut timer = interval(Duration::from_secs(10));
    let mut storage = Storage::new();

    loop {
        timer.tick().await;
        let mut vacuum = vacuum.lock().await;

        if let Ok(status) = vacuum.status().await {
            let state = prepare_state(status);

            if storage.apply_state(&state).await {
                info!("publishing state: {:?}", state);

                let topic = Topic::elisa_state();
                let payload = serde_json::to_vec(&state).unwrap();

                let message = mqtt::MessageBuilder::new()
                    .topic(topic.to_string())
                    .payload(payload)
                    .finalize();

                mqtt.publish(message).await?;
            }
        }
    }
}
