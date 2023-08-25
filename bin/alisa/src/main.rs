use alisa::{report_state, state_topics_and_qos, web_handler, ErasedError, Result};
use alisa::{Service, Topic};

use std::process;
use std::str::FromStr;
use std::time::Duration;

use futures_util::StreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::{error, info};
use paho_mqtt as mqtt;
use tokio::{task, time};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_client = connect_mqtt(mqtt_address).await?;
    info!("connected mqtt");

    let (web_handle, state_handle) = tokio::try_join!(
        task::spawn(listen_web(mqtt_client.clone())),
        task::spawn(subscribe_state(mqtt_client))
    )?;

    web_handle?;
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

async fn listen_web(mqtt: mqtt::AsyncClient) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let make_svc = make_service_fn(move |_| {
        let tx = tx.clone();
        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let tx = tx.clone();

                async move { web_handler(req, tx).await }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    let (server_handle, update_handler) = tokio::try_join!(
        task::spawn(async move {
            info!("Listening http://{}", addr);
            server.await
        }),
        task::spawn(async move {
            while let Some(payload) = rx.recv().await {
                let value = serde_json::to_vec(&payload.value)?;

                let message = mqtt::MessageBuilder::new()
                    .topic(Topic::from((Service::Elizabeth, &payload)).to_string())
                    .payload(value)
                    .finalize();

                mqtt.publish(message).await?;
            }

            Ok::<_, ErasedError>(())
        })
    )?;

    let _ = server_handle?;
    update_handler?;

    Ok(())
}

async fn subscribe_state(mut mqtt: mqtt::AsyncClient) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let (topics, qos) = state_topics_and_qos();
    mqtt.subscribe_many(&topics, &qos);

    info!("Subscribed to topics: {:?}", topics);

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            match Topic::from_str(msg.topic()) {
                Ok(topic) => match report_state(topic, msg.payload()).await {
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
