use elizabeth::{update_state, Client, Result};
use transport::elizabeth::CurrentState;
use transport::{DeviceId, DeviceType, ResponseState, Topic};

use std::process;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{debug, error, info};
use mqtt::SslOptions;
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
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password).await?;
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

async fn connect_mqtt(
    address: String,
    username: String,
    password: String,
) -> Result<mqtt::AsyncClient> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(address)
        .client_id("elizabeth")
        .finalize();

    let client = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        error!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new_v5()
        .keep_alive_interval(Duration::from_secs(30))
        .ssl_options(SslOptions::new())
        .user_name(username)
        .password(password)
        .finalize();

    let response = client.connect(conn_opts).await?;
    let response = response.connect_response().unwrap();

    debug!("client mqtt version {}", client.mqtt_version());
    debug!("server mqtt version {}", response.mqtt_version);

    Ok(client)
}

async fn subscribe_action(mut mqtt: mqtt::AsyncClient, mut inspinia: Client) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    mqtt.subscribe_many(
        &[Topic::elizabeth_action().to_string().as_str(), "request"],
        &[mqtt::QOS_1, mqtt::QOS_1],
    );
    info!("Subscribed to topic: {}", Topic::elizabeth_action());
    info!("Subscribed to topic: request");

    while let Some(msg_opt) = stream.next().await {
        debug!("got message {:?}", msg_opt);

        if let Some(msg) = msg_opt {
            match msg.topic() {
                "elizabeth/action" => match update_state(msg.payload(), &mut inspinia).await {
                    Ok(_) => (),
                    Err(err) => error!("Error updating state: {}", err),
                },
                "request" => {
                    let ids: Vec<DeviceId> = match serde_json::from_slice(msg.payload()) {
                        Ok(ids) => ids,
                        Err(err) => {
                            error!("unable to parse request: {}", err);
                            error!("{}", msg.payload_str());
                            continue;
                        }
                    };

                    let response_topic = match msg
                        .properties()
                        .get_string(mqtt::PropertyCode::ResponseTopic)
                    {
                        Some(topic) => topic,
                        None => {
                            error!("missing response topic");
                            continue;
                        }
                    };

                    let ids = ids.into_iter().filter(|id| match id.device_type {
                        DeviceType::Recuperator | DeviceType::Thermostat => true,
                        _ => false,
                    });

                    debug!("ids: {:?}", ids.clone().collect::<Vec<_>>());

                    for id in ids {
                        let capabilities =
                            inspinia.get_current_state(id.room, id.device_type).await;

                        let state = CurrentState {
                            room: id.room,
                            device_type: id.device_type,
                            capabilities,
                        };

                        debug!("publish to {}: {:?}", response_topic, state);

                        let response = ResponseState::Elizabeth(state);
                        let payload = serde_json::to_vec(&response).unwrap();

                        let message = mqtt::MessageBuilder::new()
                            .topic(&response_topic)
                            .payload(payload)
                            .finalize();

                        match mqtt.publish(message).await {
                            Ok(()) => (),
                            Err(err) => {
                                error!("Error sending response to {}: {}", response_topic, err);
                            }
                        }
                    }
                }
                _ => (),
            }
        } else {
            time::sleep(Duration::from_secs(1)).await;
            error!("Lost MQTT connection. Attempting reconnect.");
            while let Err(err) = mqtt.reconnect().await {
                error!("Error MQTT reconnecting: {}", err);
                time::sleep(Duration::from_secs(1)).await;
            }
        }
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
