use std::collections::HashMap;
use std::time::Duration;

use elisheba::{handle_action_request, handle_state_request, ErasedError, Storage};
use sonoff::{Client, Error};
use transport::state::StateUpdate;
use transport::{connect_mqtt, Topic};

use futures_util::StreamExt;
use log::{error, info, trace};
use paho_mqtt::{AsyncClient as MqClient, MessageBuilder, QOS_1};
use tokio::signal::unix::{signal, SignalKind};
use tokio::{task, time};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), ErasedError> {
    pretty_env_logger::init_timed();

    info!("elisheba version {VERSION}");

    let keys = std::env::var("KEYS").expect("set ENV variable KEYS");
    let keys: HashMap<String, String> = serde_json::from_str(&keys)?;
    let keys = keys
        .into_iter()
        .map(|(k, v)| (k, md5::compute(v).0))
        .collect();

    let client = Client::connect(keys).await?;
    info!("connected sonoff");

    client.discover().await?;

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "elisheba").await?;
    info!("connected mqtt");

    let set_handle = task::spawn(subscribe_action(mqtt_client.clone(), client.clone()));
    let state_handle = task::spawn(subscribe_state(mqtt_client, client));

    tokio::select! {
        _ = try_join(set_handle, state_handle) => {},
        _ = tokio::spawn(async move {
            let mut sig = signal(SignalKind::terminate()).unwrap();
            sig.recv().await
        }) => { info!("got SIGTERM, exiting...") },
    };

    Ok(())
}

async fn try_join(
    left: task::JoinHandle<Result<(), ErasedError>>,
    right: task::JoinHandle<Result<(), ErasedError>>,
) -> Result<(), ErasedError> {
    let (left, right) = tokio::try_join!(left, right)?;

    left?;
    right?;

    Ok(())
}

async fn subscribe_action(mut mqtt: MqClient, mut sonoff: Client) -> Result<(), ErasedError> {
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
                Topic::ActionRequest => handle_action_request(msg, &mut mqtt, &mut sonoff).await,
                Topic::StateRequest => handle_state_request(msg, &mut mqtt, &mut sonoff).await,
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

async fn subscribe_state(mqtt: MqClient, mut sonoff: Client) -> Result<(), ErasedError> {
    let mut storage = Storage::new();

    loop {
        match sonoff.read().await {
            Ok(device) => {
                let state = if let Some(state) = storage.apply(&device) {
                    state
                } else {
                    continue;
                };

                let update = StateUpdate::Elisheba(state);

                let payload = match serde_json::to_vec(&update) {
                    Ok(payload) => payload,
                    Err(err) => {
                        error!("Error serializing state update: {err}");
                        continue;
                    }
                };
                let topic = Topic::StateUpdate;

                let message = MessageBuilder::new()
                    .topic(topic.to_string())
                    .payload(payload)
                    .finalize();

                match mqtt.publish(message).await {
                    Ok(()) => (),
                    Err(err) => error!("Error publishing state update: {err}"),
                };
            }
            Err(Error::Disconnected) => {
                error!("Lost mDNS connection. Attempting reconnect.");

                while let Err(err) = sonoff.reconnect().await {
                    error!("Error mDNS reconnecting: {err}");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }

                info!("Reconnected to mDNS socket");

                sonoff.discover().await?;
            }
            Err(err) => {
                trace!("Error reading from mDNS: {err}");
            }
        }
    }
}
