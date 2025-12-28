use elisa::{handle_action_request, handle_state_request, prepare_state, Result, VacuumQueue};
use roborock::Vacuum;
use transport::state::StateUpdate;
use transport::{connect_mqtt, Topic};

use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{error, info};
use paho_mqtt::AsyncClient as MqClient;
use paho_mqtt::{MessageBuilder, QOS_1};
use tokio::{task, time};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    info!("elisa version {VERSION}");

    let vacuum_ip = std::env::var("ROBOROCK_IP")
        .unwrap_or("10.0.1.150".to_string())
        .parse()?;

    let vacuum_duid = std::env::var("ROBOROCK_DUID").expect("set ENV variable ROBOROCK_DUID");
    let vacuum_local_key =
        std::env::var("ROBOROCK_LOCAL_KEY").expect("set ENV variable ROBOROCK_LOCAL_KEY");

    let vacuum = Vacuum::new(vacuum_ip, vacuum_duid, vacuum_local_key).await?;
    let vacuum_queue = Arc::new(VacuumQueue::new(vacuum));

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "elisa").await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_actions(mqtt_client.clone(), vacuum_queue.clone())),
        task::spawn(subscribe_state(mqtt_client, vacuum_queue))
    )?;

    set_handle?;
    state_handle?;

    Ok(())
}

async fn subscribe_actions(mut mqtt: MqClient, vacuum: Arc<VacuumQueue>) -> Result<()> {
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

async fn subscribe_state(mqtt: MqClient, vacuum: Arc<VacuumQueue>) -> Result<()> {
    let mut timer = time::interval(Duration::from_secs(10));

    loop {
        timer.tick().await;
        if let Ok((status, rooms)) = vacuum.get_status().await {
            info!(
                "roborock modes: mop={:?}, water_box={:?}, wash_status={:?}, wash_phase={:?}",
                status.mop_mode, status.water_box_mode, status.wash_status, status.wash_phase
            );
            let state = prepare_state(status, &rooms);
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
