use alisa::{router, Reporter, Result};
use transport::state::StateUpdate;
use transport::{connect_mqtt, Topic};

use std::time::Duration;

use futures_util::StreamExt;
use log::{error, info};
use paho_mqtt::AsyncClient as MqClient;
use tokio::signal::unix::{signal, SignalKind};
use tokio::{task, time};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    info!("alisa version {VERSION}");

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "alisa").await?;
    info!("connected mqtt");

    let skill_id = std::env::var("ALICE_SKILL_ID").expect("skill id is required");
    let token = std::env::var("ALICE_TOKEN").expect("token is required");
    let reporter = Reporter::new(skill_id, token);

    let web_handle = task::spawn(listen_web());
    let state_handle = task::spawn(subscribe_state(mqtt_client, reporter));

    tokio::select! {
        _ = try_join(web_handle, state_handle) => {},
        _ = tokio::spawn(async move {
            let mut sig = signal(SignalKind::terminate()).unwrap();
            sig.recv().await
        }) => { info!("got SIGTERM, exiting...") },
    };

    Ok(())
}

async fn try_join(
    left: task::JoinHandle<Result<()>>,
    right: task::JoinHandle<Result<()>>,
) -> Result<()> {
    let (left, right) = tokio::try_join!(left, right)?;

    left?;
    right?;

    Ok(())
}

async fn listen_web() -> Result<()> {
    let router = router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn subscribe_state(mut mqtt: MqClient, reporter: Reporter<'_>) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let topic = Topic::StateUpdate.to_string();
    mqtt.subscribe(&topic, paho_mqtt::QOS_1).await?;
    info!("Subscribe to topic: {}", topic);

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

            loop {
                match time::timeout(Duration::from_secs(10), mqtt.reconnect()).await {
                    Ok(Ok(response)) => {
                        info!("Reconnected to MQTT! {}", response.reason_code());

                        let topic = Topic::StateUpdate.to_string();
                        mqtt.subscribe(&topic, paho_mqtt::QOS_1).await?;
                        info!("Subscribe to topic: {}", topic);

                        break;
                    }
                    Ok(Err(err)) => {
                        error!("Error MQTT reconnecting: {}", err);
                        time::sleep(Duration::from_secs(5)).await;
                    }
                    Err(err) => {
                        error!("Error MQTT reconnecting: {}", err);
                        time::sleep(Duration::from_secs(5)).await;
                    }
                }
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
