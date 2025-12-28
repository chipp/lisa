use alice::{Device, DeviceCapability, DeviceProperty, DeviceType};
use alice::{Mode, ModeFunction, Range, RangeFunction, TemperatureUnit, ToggleFunction};
use transport::{DeviceId, Room};

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Result};
use axum::Json;
use log::info;
use serde_json::json;

use crate::web_service::auth::validate_autorization;

pub async fn devices(headers: HeaderMap) -> Result<impl IntoResponse> {
    validate_autorization(&headers, "devices")?;

    let request_id = headers.get("X-Request-Id").unwrap().to_str().unwrap();
    info!("{request_id}/devices");

    let json = json!({
        "request_id": request_id,
        "payload": {
            "user_id": "chipp",
            "devices": [
                sensor_device(Room::Bedroom),
                sensor_device(Room::HomeOffice),
                sensor_device(Room::Kitchen),
                sensor_device(Room::Nursery),
                vacuum_cleaner_device(Room::Bedroom),
                vacuum_cleaner_device(Room::Corridor),
                vacuum_cleaner_device(Room::Hallway),
                vacuum_cleaner_device(Room::HomeOffice),
                vacuum_cleaner_device(Room::Kitchen),
                vacuum_cleaner_device(Room::LivingRoom),
                thermostat_device(Room::Bedroom),
                thermostat_device(Room::HomeOffice),
                thermostat_device(Room::LivingRoom),
                thermostat_device(Room::Nursery),
                recuperator_device(),
                light_device(Room::Corridor),
                light_device(Room::Nursery),
            ]
        }
    });

    Ok((StatusCode::OK, Json(json)))
}

fn name_for_room(room: &Room) -> &'static str {
    match room {
        Room::Bathroom => "Ванная",
        Room::Bedroom => "Спальня",
        Room::Corridor => "Коридор",
        Room::Hallway => "Прихожая",
        Room::HomeOffice => "Кабинет",
        Room::Kitchen => "Кухня",
        Room::LivingRoom => "Зал",
        Room::Nursery => "Детская",
        Room::Toilet => "Туалет",
    }
}

fn sensor_device(room: Room) -> Device {
    let room_name = name_for_room(&room).to_string();

    Device {
        id: DeviceId::temperature_sensor_at_room(room),
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
    let room_name = name_for_room(&room).to_string();

    Device {
        id: DeviceId::vacuum_cleaner_at_room(room),
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
            DeviceCapability::mode(
                ModeFunction::CleanupMode,
                vec![Mode::DryCleaning, Mode::MixedCleaning, Mode::WetCleaning],
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
    let room_name = name_for_room(&room).to_string();

    Device {
        id: DeviceId::thermostat_at_room(room),
        name: "Термостат".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::Thermostat,
        properties: vec![DeviceProperty::temperature().reportable()],
        capabilities: vec![
            DeviceCapability::on_off(false).reportable(),
            DeviceCapability::range(
                RangeFunction::Temperature,
                TemperatureUnit::Celsius,
                Range {
                    min: 16.0,
                    max: 28.0,
                    precision: 0.5,
                },
            )
            .reportable(),
        ],
    }
}

fn recuperator_device() -> Device {
    let room_name = name_for_room(&Room::LivingRoom).to_string();

    Device {
        id: DeviceId::recuperator_at_room(Room::LivingRoom),
        name: "Рекуператор".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::ThermostatAc,
        properties: vec![],
        capabilities: vec![
            DeviceCapability::on_off(false).reportable(),
            DeviceCapability::mode(
                ModeFunction::FanSpeed,
                vec![Mode::Low, Mode::Medium, Mode::High],
            )
            .reportable(),
        ],
    }
}

fn light_device(room: Room) -> Device {
    let room_name = name_for_room(&room).to_string();

    Device {
        id: DeviceId::light_at_room(room),
        name: "Верхний свет".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::Light,
        properties: vec![],
        capabilities: vec![DeviceCapability::on_off(false).reportable()],
    }
}
