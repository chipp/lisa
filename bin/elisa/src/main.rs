use crypto::parse_token;
use elisa::{handle_action_request, handle_state_request, prepare_state, Result, Storage};
use transport::state::StateUpdate;
use transport::{connect_mqtt, Topic};
use xiaomi::Vacuum;

use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{error, info};
use paho_mqtt::AsyncClient as MqClient;
use paho_mqtt::{MessageBuilder, QOS_1};
use tokio::sync::Mutex;
use tokio::{task, time};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    info!("elisa version {VERSION}");

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
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "elisa").await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_actions(mqtt_client.clone(), vacuum.clone())),
        task::spawn(subscribe_state(mqtt_client, vacuum))
    )?;

    set_handle?;
    state_handle?;

    Ok(())
}

async fn subscribe_actions(mut mqtt: MqClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let topics = [
        Topic::ActionRequest.to_string(),
        Topic::StateRequest.to_string(),
    ];

    mqtt.subscribe_many(&topics, &[QOS_1, QOS_1]).await?;
    info!("Subscribed to topics: {:?}", topics);

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            let topic = if let Ok(value) = msg.topic().parse() {
                value
            } else {
                continue;
            };

            match topic {
                Topic::ActionRequest => handle_action_request(msg, &mut mqtt, vacuum.clone()).await,
                Topic::StateRequest => handle_state_request(msg, &mut mqtt, vacuum.clone()).await,
                _ => (),
            }
        } else {
            time::sleep(Duration::from_secs(1)).await;
            error!("Lost MQTT connection. Attempting reconnect.");

            loop {
                match time::timeout(Duration::from_secs(10), mqtt.reconnect()).await {
                    Ok(Ok(response)) => {
                        info!("Reconnected to MQTT! {}", response.reason_code());

                        let topics = [
                            Topic::ActionRequest.to_string(),
                            Topic::StateRequest.to_string(),
                        ];

                        mqtt.subscribe_many(&topics, &[QOS_1, QOS_1]).await?;
                        info!("Subscribed to topics: {:?}", topics);

                        break;
                    }
                    Ok(Err(err)) => {
                        error!("Error MQTT reconnecting: {}", err);
                        time::sleep(Duration::from_secs(5)).await;
                    }
                    Err(err) => {
                        error!("Error MQTT reconnecting: {}", err);
                        time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn subscribe_state(mqtt: MqClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut timer = time::interval(Duration::from_secs(10));
    let mut storage = Storage::new();

    loop {
        timer.tick().await;
        let mut vacuum = vacuum.lock().await;

        if let Ok(status) = vacuum.status().await {
            let state = prepare_state(status, vacuum.last_cleaning_rooms());

            if storage.apply_state(&state) {
                info!("publishing state: {:?}", state);

                let topic = Topic::StateUpdate;

                let update = StateUpdate::Elisa(state);
                let payload = serde_json::to_vec(&update).unwrap();

                let message = MessageBuilder::new()
                    .topic(topic.to_string())
                    .payload(payload)
                    .finalize();

                match mqtt.publish(message).await {
                    Ok(()) => (),
                    Err(err) => {
                        error!("Error publishing state: {}", err);
                    }
                }
            }
        }
    }
}
