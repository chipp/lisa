pub mod elisa;
pub mod elisheba;
pub mod elizabeth;
pub mod isabel;

pub mod action {
    mod request;
    mod response;

    pub use request::Action;
    pub use request::Request as ActionRequest;
    pub use response::ActionResult;
    pub use response::Response as ActionResponse;
}

pub mod state {
    mod request;
    mod response;
    mod update;

    pub use request::Request as StateRequest;
    pub use response::Response as StateResponse;
    pub use update::Update as StateUpdate;
}

mod device_id;

pub use device_id::DeviceId;

mod topic;
pub use topic::Topic;

use log::debug;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use str_derive::Str;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Recuperator,
    TemperatureSensor,
    Thermostat,
    VacuumCleaner,
    Light,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Str, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Bathroom,
    Bedroom,
    Corridor,
    Hallway,
    HomeOffice,
    Kitchen,
    LivingRoom,
    Nursery,
    Toilet,
}

impl Room {
    pub const fn all_rooms() -> [Room; 9] {
        [
            Room::Bathroom,
            Room::Bedroom,
            Room::Corridor,
            Room::Hallway,
            Room::HomeOffice,
            Room::Kitchen,
            Room::LivingRoom,
            Room::Nursery,
            Room::Toilet,
        ]
    }
}

pub async fn connect_mqtt(
    address: String,
    username: String,
    password: String,
    client_id: &str,
) -> Result<paho_mqtt::AsyncClient, paho_mqtt::Error> {
    let create_opts = paho_mqtt::CreateOptionsBuilder::new()
        .server_uri(address)
        .client_id(client_id)
        .finalize();

    let client = paho_mqtt::AsyncClient::new(create_opts)?;

    let conn_opts = paho_mqtt::ConnectOptionsBuilder::new_v5()
        .keep_alive_interval(Duration::from_secs(30))
        .ssl_options(paho_mqtt::SslOptions::new())
        .user_name(username)
        .password(password)
        .finalize();

    let response = client.connect(conn_opts).await?;
    let response = response.connect_response().unwrap();

    debug!("client mqtt version {}", client.mqtt_version());
    debug!("server mqtt version {}", response.mqtt_version);

    Ok(client)
}
