use elizabeth::{Client, Result};
use transport::elizabeth::{Action, ActionType};
use transport::{DeviceType, Topic};

use std::process;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{debug, error, info};
use paho_mqtt as mqtt;
use tokio::task;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let inspinia_client_id =
        std::env::var("INSPINIA_CLIENT_ID").expect("set ENV variable INSPINIA_CLIENT_ID");
    let inspinia_token = std::env::var("INSPINIA_TOKEN").expect("set ENV variable INSPINIA_TOKEN");
    let inspinia_client = Client::new(inspinia_client_id, inspinia_token).await?;

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_client = connect_mqtt(mqtt_address).await?;
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

async fn subscribe_action(mut mqtt: mqtt::AsyncClient, mut inspinia: Client) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    mqtt.subscribe_many(&[Topic::elizabeth_action().to_string()], &[mqtt::QOS_1]);
    info!("Subscribed to topic: {}", Topic::elizabeth_action());

    while let Some(msg_opt) = stream.next().await {
        debug!("got message {:?}", msg_opt);

        if let Some(msg) = msg_opt {
            match update_state(msg.payload(), &mut inspinia).await {
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

async fn update_state(payload: &[u8], inspinia: &mut Client) -> Result<()> {
    let action: Action = serde_json::from_slice(payload)?;

    debug!("Action: {:?}", action);

    match (action.device_type, action.action_type) {
        (DeviceType::Recuperator, ActionType::SetIsEnabled(value)) => {
            inspinia.set_recuperator_enabled(value).await?;
        }
        (DeviceType::Recuperator, ActionType::SetFanSpeed(speed)) => {
            inspinia
                .set_recuperator_fan_speed(map_fan_speed(speed))
                .await?;
        }
        (DeviceType::Thermostat, ActionType::SetIsEnabled(value)) => {
            if let Some(room) = map_room(action.room) {
                inspinia.set_thermostat_enabled(value, room).await?;
            }
        }
        (DeviceType::Thermostat, ActionType::SetTemperature(value, relative)) => {
            if let Some(room) = map_room(action.room) {
                if relative {
                    let current = inspinia.get_thermostat_temperature_in_room(room).await?;

                    debug!("current: {}", current);
                    debug!("value: {}", value);

                    inspinia
                        .set_thermostat_temperature(current + value, room)
                        .await?;
                } else {
                    inspinia.set_thermostat_temperature(value, room).await?;
                }
            }
        }
        _ => (),
    }

    Ok(())
}

async fn subscribe_state(mqtt: mqtt::AsyncClient, mut inspinia: Client) -> Result<()> {
    loop {
        if let Ok(payload) = inspinia.read().await {
            let payload = serde_json::to_vec(&payload)?;
            let topic = Topic::elizabeth_state();

            let message = mqtt::MessageBuilder::new()
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

fn map_room(room: transport::Room) -> Option<inspinia::Room> {
    match room {
        transport::Room::LivingRoom => Some(inspinia::Room::LivingRoom),
        transport::Room::Bedroom => Some(inspinia::Room::Bedroom),
        transport::Room::HomeOffice => Some(inspinia::Room::HomeOffice),
        transport::Room::Nursery => Some(inspinia::Room::Nursery),
        _ => None,
    }
}

fn map_fan_speed(speed: transport::elizabeth::FanSpeed) -> inspinia::FanSpeed {
    match speed {
        transport::elizabeth::FanSpeed::Low => inspinia::FanSpeed::Low,
        transport::elizabeth::FanSpeed::Medium => inspinia::FanSpeed::Medium,
        transport::elizabeth::FanSpeed::High => inspinia::FanSpeed::High,
    }
}
