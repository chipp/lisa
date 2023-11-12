use elizabeth::{update_state, Client, Result};
use transport::Topic;

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
    let create_opts = mqtt::CreateOptionsBuilder::new_v3()
        .server_uri(address)
        .client_id("elizabeth")
        .finalize();

    let client = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        error!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new_v3()
        .keep_alive_interval(Duration::from_secs(30))
        .clean_session(false)
        .ssl_options(SslOptions::new())
        .user_name(username)
        .password(password)
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
