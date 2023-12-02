use alisa::{web_handler, ErasedError, Reporter, Result};
use transport::state::StateUpdate;
use transport::Topic;

use std::process;
use std::time::Duration;

use futures_util::StreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::{debug, error, info};
use mqtt::SslOptions;
use paho_mqtt as mqtt;
use tokio::{task, time};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password).await?;
    info!("connected mqtt");

    let skill_id = std::env::var("ALICE_SKILL_ID").expect("skill id is required");
    let token = std::env::var("ALICE_TOKEN").expect("token is required");
    let reporter = Reporter::new(skill_id, token);

    let (web_handle, state_handle) = tokio::try_join!(
        task::spawn(listen_web(mqtt_client.clone())),
        task::spawn(subscribe_state(mqtt_client, reporter))
    )?;

    web_handle?;
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
        .client_id("alisa")
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

async fn listen_web(mqtt: mqtt::AsyncClient) -> Result<()> {
    let make_svc = make_service_fn(move |_| {
        let mqtt = mqtt.clone();

        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let mqtt = mqtt.clone();
                async move { web_handler(req, mqtt).await }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening http://{}", addr);
    server.await?;

    Ok(())
}

async fn subscribe_state(mut mqtt: mqtt::AsyncClient, reporter: Reporter) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let topic = Topic::State.to_string();
    info!("Subscribe to topic: {}", topic);
    mqtt.subscribe(topic, mqtt::QOS_1);

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            if let Some(event) = parse_update(&msg) {
                match reporter.report_update(event).await {
                    Ok(_) => (),
                    Err(err) => error!("Error updating state: {}", err),
                }
            } else {
                error!("unable to parse topic {}", msg.topic());
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

fn parse_update(msg: &mqtt::Message) -> Option<StateUpdate> {
    if let Topic::State = msg.topic().parse().ok()? {
        let update: StateUpdate = serde_json::from_slice(msg.payload()).ok()?;
        Some(update)
    } else {
        None
    }
}
