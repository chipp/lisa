use alice::{
    Device, DeviceCapability, DeviceProperty, DeviceType, Mode, ModeFunction, ToggleFunction,
};
use hyper::{Body, Request, Response, StatusCode};
use serde_json::json;

use super::super::auth::validate_autorization;
use crate::{DeviceId, Result, Room};

pub async fn devices(request: Request<Body>) -> Result<Response<Body>> {
    validate_autorization(request, "devices", |request| async move {
        let request_id =
            std::str::from_utf8(request.headers().get("X-Request-Id").unwrap().as_bytes()).unwrap();

        let json = json!({
            "request_id": request_id,
            "payload": {
                "user_id": "chipp",
                "devices": [
                    sensor_device(Room::Bedroom),
                    sensor_device(Room::LivingRoom),
                    sensor_device(Room::Nursery),
                    vacuum_cleaner_device(Room::Bathroom),
                    vacuum_cleaner_device(Room::Bedroom),
                    vacuum_cleaner_device(Room::Corridor),
                    vacuum_cleaner_device(Room::Hallway),
                    vacuum_cleaner_device(Room::HomeOffice),
                    vacuum_cleaner_device(Room::Kitchen),
                    vacuum_cleaner_device(Room::LivingRoom),
                    vacuum_cleaner_device(Room::Nursery),
                    vacuum_cleaner_device(Room::Toilet),
                ]
            }
        });

        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(serde_json::to_vec(&json)?))?)
    })
    .await
}

fn sensor_device(room: Room) -> Device {
    let room_name = room.name().to_string();

    Device {
        id: DeviceId::temperature_sensor_at_room(room).to_string(),
        name: "Датчик температуры".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::Sensor,
        properties: vec![
            DeviceProperty::humidity().retrievable().reportable(),
            DeviceProperty::temperature().retrievable().reportable(),
            DeviceProperty::battery_level().retrievable().reportable(),
        ],
        capabilities: vec![],
    }
}

fn vacuum_cleaner_device(room: Room) -> Device {
    let room_name = room.name().to_string();

    Device {
        id: DeviceId::vacuum_cleaner_at_room(room).to_string(),
        name: "Джордан".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::VacuumCleaner,
        properties: vec![DeviceProperty::battery_level().retrievable().reportable()],
        capabilities: vec![
            DeviceCapability::on_off(false).retrievable().reportable(),
            DeviceCapability::mode(
                ModeFunction::WorkSpeed,
                vec![Mode::Quiet, Mode::Normal, Mode::Medium, Mode::Turbo],
            )
            .retrievable()
            .reportable(),
            DeviceCapability::toggle(ToggleFunction::Pause)
                .retrievable()
                .reportable(),
        ],
    }
}
