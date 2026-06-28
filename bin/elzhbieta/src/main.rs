use std::sync::Arc;
use std::time::Duration;

use elzhbieta::{
    handle_action_request, handle_state_request, publish_all_states, Result, SharedClient,
};
use futures_util::StreamExt;
use log::{error, info};
use paho_mqtt::{AsyncClient as MqClient, QOS_1};
use tokio::sync::Mutex;
use tokio::{task, time};
use transport::{connect_mqtt, Topic};
use tuya::DeviceConfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    info!("Elžbieta Tuya AC gateway version {VERSION}");

    let devices = std::env::var("ELZHBIETA_DEVICES").expect("set ENV variable ELZHBIETA_DEVICES");
    let devices: Vec<DeviceConfig> = serde_json::from_str(&devices)?;
    let client = tuya::Client::connect(devices).await?;
    let client = Arc::new(Mutex::new(client));
    info!("connected tuya devices");

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "elzhbieta").await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_requests(mqtt_client.clone(), client.clone())),
        task::spawn(publish_state_loop(mqtt_client, client))
    )?;

    set_handle?;
    state_handle?;

    Ok(())
}

async fn subscribe_requests(mut mqtt: MqClient, client: SharedClient) -> Result<()> {
    let stream = mqtt.get_stream(None);
    futures_util::pin_mut!(stream);

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
                Topic::ActionRequest => handle_action_request(msg, &mut mqtt, client.clone()).await,
                Topic::StateRequest => handle_state_request(msg, &mut mqtt, client.clone()).await,
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

async fn publish_state_loop(mqtt: MqClient, client: SharedClient) -> Result<()> {
    let mut timer = time::interval(Duration::from_secs(10));

    loop {
        timer.tick().await;
        publish_all_states(&mqtt, client.clone()).await;
    }
}
