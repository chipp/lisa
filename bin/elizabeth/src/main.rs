use elizabeth::set_topics_and_qos;
use elizabeth::{InspiniaClient, Result, State, StatePayload};
use inspinia::FanSpeed;
use topics::{Device, Topic};

use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{error, info};
use paho_mqtt as mqtt;
use serde_json::json;
use tokio::time;
use tokio::{sync::Mutex, task};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let inspinia_token = std::env::var("INSPINIA_TOKEN").expect("set ENV variable INSPINIA_TOKEN");
    let inspinia_client = Arc::from(Mutex::from(InspiniaClient::new(inspinia_token).await?));

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_client = connect_mqtt(mqtt_address).await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_set(mqtt_client.clone(), inspinia_client.clone())),
        task::spawn(subscribe_state(mqtt_client, inspinia_client))
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

async fn subscribe_set(
    mut mqtt: mqtt::AsyncClient,
    inspinia: Arc<Mutex<InspiniaClient>>,
) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

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

async fn update_state(
    topic: Topic<State>,
    payload: &[u8],
    inspinia: &mut InspiniaClient,
) -> Result<()> {
    let value: serde_json::Value = serde_json::from_slice(payload).unwrap();

    match (topic.device, topic.feature) {
        (Device::Recuperator, State::IsEnabled) => {
            if let Some(value) = value.as_bool() {
                inspinia.set_recuperator_enabled(value).await?;
            }
        }
        (Device::Recuperator, State::FanSpeed) => {
            if let Some(value) = value.as_str() {
                let value = FanSpeed::try_from(value)?;
                inspinia.set_recuperator_fan_speed(value).await?;
            }
        }
        (Device::Thermostat, State::IsEnabled) => {
            if let Some(value) = value.as_bool() {
                if let Some(room) = topic.room.and_then(map_room) {
                    inspinia.set_thermostat_enabled(value, room).await?;
                }
            }
        }
        (Device::Thermostat, State::Temperature) => {
            if let Some(value) = value.as_f64() {
                if let Some(room) = topic.room.and_then(map_room) {
                    inspinia.set_thermostat_temperature(value, room).await?;
                }
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
                let topic: Topic<State> = payload.into();

                let message = mqtt::MessageBuilder::new()
                    .topic(topic.to_string())
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
    match payload.state {
        State::IsEnabled => Some(json!(payload.value == "1")),
        State::FanSpeed => Some(json!(payload.value.to_lowercase())),
        State::CurrentTemperature | State::Temperature => {
            let value = f32::from_str(&payload.value).ok()?;
            Some(json!(value))
        }
        State::Mode => None,
    }
}

fn map_room(room: topics::Room) -> Option<inspinia::Room> {
    match room {
        topics::Room::LivingRoom => Some(inspinia::Room::LivingRoom),
        topics::Room::Bedroom => Some(inspinia::Room::Bedroom),
        topics::Room::HomeOffice => Some(inspinia::Room::HomeOffice),
        topics::Room::Nursery => Some(inspinia::Room::Nursery),
        _ => None,
    }
}
