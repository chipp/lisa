mod client;
pub use client::Client;

use log::debug;
use transport::{
    elizabeth::{Action, ActionType},
    DeviceType,
};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

pub async fn update_state(payload: &[u8], inspinia: &mut Client) -> Result<()> {
    let action: Action = serde_json::from_slice(payload)?;

    debug!("Action: {:?}", action);

    match (action.device_type, action.action_type) {
        (DeviceType::Recuperator, ActionType::SetIsEnabled(value)) => {
            inspinia.set_recuperator_enabled(value).await?;
        }
        (DeviceType::Recuperator, ActionType::SetFanSpeed(speed)) => {
            inspinia
                .set_recuperator_fan_speed(from_elizabeth_speed(speed))
                .await?;
        }
        (DeviceType::Thermostat, ActionType::SetIsEnabled(value)) => {
            if let Some(room) = from_elizabeth_room(action.room) {
                inspinia.set_thermostat_enabled(value, room).await?;
            }
        }
        (DeviceType::Thermostat, ActionType::SetTemperature(value, relative)) => {
            if let Some(room) = from_elizabeth_room(action.room) {
                if relative {
                    let current = inspinia.get_thermostat_temperature_in_room(room).await?;

                    debug!("current: {}", current);
                    debug!("value: {}", value);

                    inspinia
                        .set_thermostat_temperature(current + value, room)
                        .await?;
                } else {
                    inspinia.set_thermostat_temperature(value, room).await?;
                }
            }
        }
        _ => (),
    }

    Ok(())
}

fn from_elizabeth_room(room: transport::Room) -> Option<inspinia::Room> {
    match room {
        transport::Room::LivingRoom => Some(inspinia::Room::LivingRoom),
        transport::Room::Bedroom => Some(inspinia::Room::Bedroom),
        transport::Room::HomeOffice => Some(inspinia::Room::HomeOffice),
        transport::Room::Nursery => Some(inspinia::Room::Nursery),
        _ => None,
    }
}

fn from_elizabeth_speed(speed: transport::elizabeth::FanSpeed) -> inspinia::FanSpeed {
    match speed {
        transport::elizabeth::FanSpeed::Low => inspinia::FanSpeed::Low,
        transport::elizabeth::FanSpeed::Medium => inspinia::FanSpeed::Medium,
        transport::elizabeth::FanSpeed::High => inspinia::FanSpeed::High,
    }
}
