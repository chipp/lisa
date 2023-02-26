use elizabeth::{set_topics_and_qos, DeviceType, Topic};
use elizabeth::{Capability, InspiniaClient, Result, StatePayload};
use inspinia::FanSpeed;
use tokio::time;

use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{error, info};
use paho_mqtt as mqtt;
use serde_json::json;
use tokio::{sync::Mutex, task};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let token = std::env::var("INSPINIA_TOKEN").expect("set ENV variable INSPINIA_TOKEN");
    let inspinia_client = Arc::from(Mutex::from(InspiniaClient::new(token).await?));

    let mqtt_client = connect_mqtt().await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_set(mqtt_client.clone(), inspinia_client.clone())),
        task::spawn(subscribe_state(mqtt_client, inspinia_client))
    )?;

    set_handle?;
    state_handle?;

    Ok(())
}

async fn connect_mqtt() -> Result<mqtt::AsyncClient> {
    let client = mqtt::AsyncClient::new("mqtt://localhost:1883").unwrap_or_else(|err| {
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

async fn subscribe_set(
    mut mqtt: mqtt::AsyncClient,
    inspinia: Arc<Mutex<InspiniaClient>>,
) -> Result<()> {
    let mut stream = mqtt.get_stream(5);

    let (topics, qos) = set_topics_and_qos();
    mqtt.subscribe_many(&topics, &qos);

    info!("Subscribed to topics: {:?}", topics);

    while let Some(msg_opt) = stream.next().await {
        let inspinia = &mut inspinia.lock().await;

        if let Some(msg) = msg_opt {
            match Topic::from_str(msg.topic()) {
                Ok(topic) => match update_state(topic, msg.payload(), inspinia).await {
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

async fn update_state(topic: Topic, payload: &[u8], inspinia: &mut InspiniaClient) -> Result<()> {
    let value: serde_json::Value = serde_json::from_slice(payload).unwrap();

    match (topic.device_type, topic.capability) {
        (DeviceType::Recuperator, Capability::IsEnabled) => {
            if let Some(value) = value.as_bool() {
                inspinia.set_recuperator_enabled(value).await?;
            }
        }
        (DeviceType::Recuperator, Capability::FanSpeed) => {
            if let Some(value) = value.as_str() {
                let value = FanSpeed::try_from(value)?;
                inspinia.set_recuperator_fan_speed(value).await?;
            }
        }
        (DeviceType::Thermostat, Capability::IsEnabled) => {
            if let Some(value) = value.as_bool() {
                inspinia.set_thermostat_enabled(value, topic.room).await?;
            }
        }
        (DeviceType::Thermostat, Capability::Temperature) => {
            if let Some(value) = value.as_f64() {
                inspinia
                    .set_thermostat_temperature(value, topic.room)
                    .await?;
            }
        }
        _ => (),
    }

    Ok(())
}

async fn subscribe_state(
    mqtt: mqtt::AsyncClient,
    inspinia: Arc<Mutex<InspiniaClient>>,
) -> Result<()> {
    loop {
        let inspinia = &mut inspinia.lock().await;

        if let Ok(payload) = inspinia.read().await {
            if let Some(value) = value_for_payload(&payload) {
                let value = serde_json::to_vec(&value)?;

                let message = mqtt::MessageBuilder::new()
                    .topic(Topic::from(&payload).to_string())
                    .payload(value)
                    .finalize();

                mqtt.publish(message).await?;
            }
        } else {
            error!("Lost Inspinia connection. Attempting reconnect.");

            while let Err(err) = inspinia.reconnect().await {
                error!("Error Inspinia reconnecting: {}", err);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn value_for_payload(payload: &StatePayload) -> Option<serde_json::Value> {
    match payload.capability {
        Capability::IsEnabled => Some(json!(payload.value == "1")),
        Capability::FanSpeed => Some(json!(payload.value)),
        Capability::CurrentTemperature | Capability::Temperature => {
            let value = f32::from_str(&payload.value).ok()?;
            Some(json!(value))
        }
        Capability::Mode => None,
    }
}
