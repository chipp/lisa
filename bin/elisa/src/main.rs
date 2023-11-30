use elisa::{
    handle_action_request, handle_state_request, perform_action, prepare_state, Result, Storage,
};
use transport::Topic;
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

    let topics = [
        Topic::ActionRequest.to_string(),
        Topic::StateRequest.to_string(),
    ];

    mqtt.subscribe_many(&topics, &[mqtt::QOS_1, mqtt::QOS_1]);
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

                let topic = Topic::State;
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
