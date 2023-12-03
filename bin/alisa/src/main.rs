use alisa::{web_handler, ErasedError, Reporter, Result};
use transport::state::StateUpdate;
use transport::{connect_mqtt, Topic};

use std::time::Duration;

use futures_util::StreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::{error, info};
use paho_mqtt::AsyncClient as MqClient;
use tokio::{task, time};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "alisa").await?;
    info!("connected mqtt");

    let skill_id = std::env::var("ALICE_SKILL_ID").expect("skill id is required");
    let token = std::env::var("ALICE_TOKEN").expect("token is required");
    let reporter = Reporter::new(skill_id, token);

    let (web_handle, state_handle) = tokio::try_join!(
        task::spawn(listen_web()),
        task::spawn(subscribe_state(mqtt_client, reporter))
    )?;

    web_handle?;
    state_handle?;

    Ok(())
}

async fn listen_web() -> Result<()> {
    let make_svc = make_service_fn(move |_| async move {
        Ok::<_, ErasedError>(service_fn(move |req| async move { web_handler(req).await }))
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening http://{}", addr);
    server.await?;

    Ok(())
}

async fn subscribe_state(mut mqtt: MqClient, reporter: Reporter) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let topic = Topic::StateUpdate.to_string();
    info!("Subscribe to topic: {}", topic);
    mqtt.subscribe(topic, paho_mqtt::QOS_1);

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

fn parse_update(msg: &paho_mqtt::Message) -> Option<StateUpdate> {
    if let Topic::StateUpdate = msg.topic().parse().ok()? {
        let update: StateUpdate = serde_json::from_slice(msg.payload()).ok()?;
        Some(update)
    } else {
        None
    }
}
