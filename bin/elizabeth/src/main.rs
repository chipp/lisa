use elizabeth::{handle_action_request, handle_state_request, Client, Result};
use transport::state::StateUpdate;
use transport::Topic;

use std::process;
use std::time::Duration;

use futures_util::stream::StreamExt;
use log::{debug, error, info, trace};
use paho_mqtt::AsyncClient as MqClient;
use paho_mqtt::{ConnectOptionsBuilder, CreateOptionsBuilder, MessageBuilder, SslOptions, QOS_1};
use tokio::task;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

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

async fn connect_mqtt(address: String, username: String, password: String) -> Result<MqClient> {
    let create_opts = CreateOptionsBuilder::new()
        .server_uri(address)
        .client_id("elizabeth")
        .finalize();

    let client = MqClient::new(create_opts).unwrap_or_else(|err| {
        error!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = ConnectOptionsBuilder::new_v5()
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

async fn subscribe_action(mut mqtt: MqClient, mut inspinia: Client) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let topics = [
        Topic::ActionRequest.to_string(),
        Topic::StateRequest.to_string(),
    ];

    mqtt.subscribe_many(&topics, &[QOS_1, QOS_1]);
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
            while let Err(err) = mqtt.reconnect().await {
                error!("Error MQTT reconnecting: {}", err);
                time::sleep(Duration::from_secs(1)).await;
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
            let topic = Topic::State;

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
