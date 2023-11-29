use alisa::{report_state, web_handler, Action, ErasedError, Result, State};
use transport::elisa::State as ElisaState;
use transport::elizabeth::State as ElizabethState;
use transport::{Service, Topic, TopicType};

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

    let (web_handle, state_handle) = tokio::try_join!(
        task::spawn(listen_web(mqtt_client.clone())),
        task::spawn(subscribe_state(mqtt_client))
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
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Action>();
    let mqtt1 = mqtt.clone();

    let make_svc = make_service_fn(move |_| {
        let tx = tx.clone();
        let mqtt = mqtt1.clone();

        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let tx = tx.clone();
                let mqtt = mqtt.clone();
                async move { web_handler(req, tx, mqtt).await }
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
            while let Some(action) = rx.recv().await {
                // TODO: handle errors

                let (topic, value) = match action {
                    Action::Elizabeth(action) => (
                        Topic::elizabeth_action(),
                        serde_json::to_vec(&action).unwrap(),
                    ),
                    Action::Elisa(action) => {
                        (Topic::elisa_action(), serde_json::to_vec(&action).unwrap())
                    }
                };

                let message = mqtt::MessageBuilder::new()
                    .topic(topic.to_string())
                    .payload(value)
                    .finalize();

                mqtt.publish(message).await?;
            }

            Ok::<_, ErasedError>(())
        })
    )?;

    server_handle?;
    update_handler?;

    Ok(())
}

async fn subscribe_state(mut mqtt: mqtt::AsyncClient) -> Result<()> {
    let mut stream = mqtt.get_stream(None);

    let topics = &[
        Topic::elisa_state().to_string(),
        Topic::elizabeth_state().to_string(),
    ];
    let qos = &[mqtt::QOS_1; 2];

    mqtt.subscribe_many(topics, qos);

    info!("Subscribed to topics: {:?}", topics);

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            if let Some(event) = parse_state(&msg) {
                match report_state(event).await {
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

fn parse_state(msg: &mqtt::Message) -> Option<State> {
    let topic: Topic = msg.topic().parse().ok()?;

    if let TopicType::State = topic.topic_type {
        match topic.service {
            Service::Elizabeth => {
                let state: ElizabethState = serde_json::from_slice(msg.payload()).ok()?;
                Some(State::Elizabeth(state))
            }
            Service::Elisa => {
                let state: ElisaState = serde_json::from_slice(msg.payload()).ok()?;
                Some(State::Elisa(state))
            }
        }
    } else {
        None
    }
}
