use bluetooth::{Event, MacAddr, Scanner, ScannerTrait};
use isabel::{Result, Storage};
use transport::{
    connect_mqtt,
    isabel::{Property, State},
    state::StateUpdate,
    Room, Topic,
};

use std::time::Duration;

use log::{debug, error, info};
use paho_mqtt::AsyncClient as MqClient;
use paho_mqtt::MessageBuilder;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mqtt_address = std::env::var("MQTT_ADDRESS").expect("set ENV variable MQTT_ADDRESS");
    let mqtt_username = std::env::var("MQTT_USER").expect("set ENV variable MQTT_USER");
    let mqtt_password = std::env::var("MQTT_PASS").expect("set ENV variable MQTT_PASS");
    let mqtt_client = connect_mqtt(mqtt_address, mqtt_username, mqtt_password, "isabel").await?;
    info!("connected mqtt");

    subscribe_state(mqtt_client).await?;

    Ok(())
}

async fn subscribe_state(mqtt: MqClient) -> Result<()> {
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
    let mut storage = Storage::default();

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

            let state = State { room, property };
            if !storage.apply_update(&state) {
                continue;
            }

            let topic = Topic::StateUpdate;

            debug!("sending state {:?}", state);

            let update = StateUpdate::Isabel(state);

            let payload = serde_json::to_vec(&update).unwrap();

            let message = MessageBuilder::new()
                .topic(topic.to_string())
                .payload(payload)
                .finalize();

            match mqtt.publish(message).await {
                Ok(()) => (),
                Err(err) => {
                    error!("Error publishing state: {}", err);

                    if !mqtt.is_connected() {
                        time::sleep(Duration::from_secs(1)).await;
                        error!("Lost MQTT connection. Attempting reconnect.");
                        while let Err(err) = mqtt.reconnect().await {
                            error!("Error MQTT reconnecting: {}", err);
                            time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
