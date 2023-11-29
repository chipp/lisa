use elisa::{perform_action, prepare_state, Result, Storage};
use transport::{DeviceId, DeviceType, ResponseState, Topic};
use xiaomi::{parse_token, Vacuum};

use std::process;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{debug, error, info};
use mqtt::SslOptions;
use paho_mqtt as mqtt;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{self, interval};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

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
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password).await?;
    info!("connected mqtt");

    let (set_handle, state_handle) = tokio::try_join!(
        task::spawn(subscribe_actions(mqtt_client.clone(), vacuum.clone())),
        task::spawn(subscribe_state(mqtt_client, vacuum))
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
        .client_id("elisa")
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

async fn subscribe_actions(mut mqtt: mqtt::AsyncClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    mqtt.subscribe_many(
        &[Topic::elisa_action().to_string().as_str(), "request"],
        &[mqtt::QOS_1, mqtt::QOS_1],
    );

    info!("Subscribed to topic: {}", Topic::elisa_action());
    info!("Subscribed to topic: request");

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            match msg.topic() {
                "elisa/action" => {
                    let vacuum = &mut vacuum.lock().await;
                    match perform_action(msg.payload(), vacuum).await {
                        Ok(_) => (),
                        Err(err) => error!("Error updating state: {}", err),
                    }
                }
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

                    let should_respond = ids
                        .iter()
                        .any(|id| id.device_type == DeviceType::VacuumCleaner);

                    if should_respond {
                        let mut vacuum = vacuum.lock().await;

                        if let Ok(status) = vacuum.status().await {
                            let state = prepare_state(status, vacuum.last_cleaning_rooms());
                            debug!("publish to {}: {:?}", response_topic, state);

                            let response = ResponseState::Elisa(state);

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

async fn subscribe_state(mqtt: mqtt::AsyncClient, vacuum: Arc<Mutex<Vacuum>>) -> Result<()> {
    let mut timer = interval(Duration::from_secs(10));
    let mut storage = Storage::new();

    loop {
        timer.tick().await;
        let mut vacuum = vacuum.lock().await;

        if let Ok(status) = vacuum.status().await {
            let state = prepare_state(status, vacuum.last_cleaning_rooms());

            if storage.apply_state(&state).await {
                info!("publishing state: {:?}", state);

                let topic = Topic::elisa_state();
                let payload = serde_json::to_vec(&state).unwrap();

                let message = mqtt::MessageBuilder::new()
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
