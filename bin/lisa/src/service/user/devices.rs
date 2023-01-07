use alice::{Device, DeviceCapability, DeviceProperty, DeviceType};
use alice::{Mode, ModeFunction, Range, RangeFunction, TemperatureUnit, ToggleFunction};

use hyper::{Body, Request, Response, StatusCode};
use serde_json::json;

use super::super::auth::validate_autorization;
use crate::{DeviceId, Result, Room};

pub async fn devices(request: Request<Body>) -> Result<Response<Body>> {
    validate_autorization(request, "devices", |request| async move {
        let request_id =
            std::str::from_utf8(request.headers().get("X-Request-Id").unwrap().as_bytes()).unwrap();

        let mut devices = vec![
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
        ];

        if cfg!(feature = "inspinia") {
            devices.push(thermostat_device(Room::Bedroom));
            devices.push(thermostat_device(Room::HomeOffice));
            devices.push(thermostat_device(Room::LivingRoom));
            devices.push(thermostat_device(Room::Nursery));
            devices.push(recuperator_device());
        }

        let json = json!({
            "request_id": request_id,
            "payload": {
                "user_id": "chipp",
                "devices": devices
            }
        });

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
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
            DeviceProperty::humidity().reportable(),
            DeviceProperty::temperature().reportable(),
            DeviceProperty::battery_level().reportable(),
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

fn thermostat_device(room: Room) -> Device {
    let room_name = room.name().to_string();

    Device {
        id: DeviceId::thermostat_at_room(room).to_string(),
        name: "Термостат".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::Thermostat,
        properties: vec![DeviceProperty::temperature().reportable()],
        capabilities: vec![
            DeviceCapability::on_off(false).retrievable().reportable(),
            DeviceCapability::range(
                RangeFunction::Temperature,
                TemperatureUnit::Celsius,
                Range {
                    min: 16.0,
                    max: 28.0,
                    precision: 0.5,
                },
            )
            .retrievable()
            .reportable(),
        ],
    }
}

fn recuperator_device() -> Device {
    let room_name = Room::LivingRoom.name().to_string();

    Device {
        id: DeviceId::recuperator_at_room(Room::LivingRoom).to_string(),
        name: "Рекуператор".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::ThermostatAc,
        properties: vec![],
        capabilities: vec![
            DeviceCapability::on_off(false).retrievable().reportable(),
            DeviceCapability::mode(
                ModeFunction::WorkSpeed,
                vec![Mode::Low, Mode::Medium, Mode::High],
            )
            .retrievable()
            .reportable(),
        ],
    }
}
