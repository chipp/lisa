use bluetooth::{Event, MacAddr, Scanner, ScannerTrait};

use isabel::Result;

use transport::{
    isabel::{Property, State},
    state::StateUpdate,
    Room, Topic,
};

use std::process;
use std::time::Duration;

use log::{debug, error, info};
use mqtt::SslOptions;
use paho_mqtt as mqtt;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password).await?;
    info!("connected mqtt");

    subscribe_state(mqtt_client).await?;

    Ok(())
}

async fn connect_mqtt(
    address: String,
    username: String,
    password: String,
) -> Result<mqtt::AsyncClient> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(address)
        .client_id("isabel")
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

async fn subscribe_state(mqtt: mqtt::AsyncClient) -> Result<()> {
    let mut scanner = Scanner::new();

    fn match_addr_to_room(addr: MacAddr) -> Option<Room> {
        match addr.octets {
            [0x58, 0x2d, 0x34, 0x39, 0x95, 0xf2] => Some(Room::Bedroom),
            [0x4c, 0x65, 0xa8, 0xdd, 0x82, 0xcf] => Some(Room::HomeOffice),
            [0x58, 0x2d, 0x34, 0x39, 0x97, 0x66] => Some(Room::Kitchen),
            [0x58, 0x2d, 0x34, 0x36, 0x32, 0x9b] => Some(Room::Nursery),
            _ => None,
        }
    }

    let mut rx = scanner.start_scan();

    while let Some((addr, event)) = rx.recv().await {
        if let Some(room) = match_addr_to_room(addr) {
            let property = match event {
                Event::Temperature(temperature) => Property::Temperature(temperature as f32 / 10.0),
                Event::Humidity(humidity) => Property::Humidity(humidity as f32 / 10.0),
                Event::Battery(battery) => Property::Battery(battery),
                Event::TemperatureAndHumidity(temperature, humidity) => {
                    Property::TemperatureAndHumidity(
                        temperature as f32 / 10.0,
                        humidity as f32 / 10.0,
                    )
                }
            };

            let topic = Topic::State;
            let state = State { room, property };

            debug!("sending state {:?}", state);

            let update = StateUpdate::Isabel(state);

            let payload = serde_json::to_vec(&update).unwrap();

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

    Ok(())
}
