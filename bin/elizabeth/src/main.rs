use elizabeth::{handle_action_request, handle_state_request, Client, Result};
use transport::state::StateUpdate;
use transport::{connect_mqtt, Topic};

use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{error, info, trace};
use paho_mqtt::AsyncClient as MqClient;
use paho_mqtt::{MessageBuilder, QOS_1};
use tokio::{task, time};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    info!("elizabeth version {VERSION}");

    let inspinia_client_id =
        std::env::var("INSPINIA_CLIENT_ID").expect("set ENV variable INSPINIA_CLIENT_ID");
    let inspinia_token = std::env::var("INSPINIA_TOKEN").expect("set ENV variable INSPINIA_TOKEN");
    let inspinia_client = Client::new(inspinia_client_id, inspinia_token).await?;

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "elizabeth").await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_action(
            mqtt_client.clone(),
            inspinia_client.clone()
        )),
        task::spawn(subscribe_state(mqtt_client, inspinia_client))
    )?;

    set_handle?;
    state_handle?;

    Ok(())
}

async fn subscribe_action(mut mqtt: MqClient, mut inspinia: Client) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let topics = [
        Topic::ActionRequest.to_string(),
        Topic::StateRequest.to_string(),
    ];

    mqtt.subscribe_many(&topics, &[QOS_1, QOS_1]).await?;
    info!("Subscribed to topis: {:?}", topics);

    while let Some(msg_opt) = stream.next().await {
        trace!("got message {:?}", msg_opt);

        if let Some(msg) = msg_opt {
            let topic = if let Ok(value) = msg.topic().parse() {
                value
            } else {
                continue;
            };

            match topic {
                Topic::ActionRequest => handle_action_request(msg, &mut mqtt, &mut inspinia).await,
                Topic::StateRequest => handle_state_request(msg, &mut mqtt, &mut inspinia).await,
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
                        info!("Subscribed to topis: {:?}", topics);

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

async fn subscribe_state(mqtt: MqClient, mut inspinia: Client) -> Result<()> {
    loop {
        if let Ok(payload) = inspinia.read().await {
            let update = StateUpdate::Elizabeth(payload);

            let payload = serde_json::to_vec(&update)?;
            let topic = Topic::StateUpdate;

            let message = MessageBuilder::new()
                .topic(topic.to_string())
                .payload(payload)
                .finalize();

            mqtt.publish(message).await?;
        } else {
            error!("Lost Inspinia connection. Attempting reconnect.");

            while let Err(err) = inspinia.reconnect().await {
                error!("Error Inspinia reconnecting: {}", err);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
