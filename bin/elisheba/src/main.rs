use std::collections::HashMap;
use std::time::Duration;

use elisheba::Storage;
use sonoff::{Client, Error};
use transport::state::StateUpdate;
use transport::{connect_mqtt, Topic};

use log::{error, info, trace};
use paho_mqtt::{AsyncClient as MqClient, MessageBuilder};
use tokio::signal::unix::{signal, SignalKind};
use tokio::task;

type ErasedError = Box<dyn std::error::Error + Send + Sync + 'static>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), ErasedError> {
    pretty_env_logger::init_timed();

    info!("elisheba version {VERSION}");

    let devices = std::env::var("DEVICES").expect("set ENV variable DEVICES");
    let devices: HashMap<String, String> = serde_json::from_str(&devices)?;
    let devices = devices
        .into_iter()
        .map(|(k, v)| (k, md5::compute(v).0))
        .collect();

    let client = Client::connect(devices).await?;
    info!("connected sonoff");

    client.discover().await?;

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "elisheba").await?;
    info!("connected mqtt");

    let state_handle = task::spawn(subscribe_state(mqtt_client, client));

    tokio::select! {
        _ = state_handle => {},
        _ = tokio::spawn(async move {
            let mut sig = signal(SignalKind::terminate()).unwrap();
            sig.recv().await
        }) => { info!("got SIGTERM, exiting...") },
    };

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
