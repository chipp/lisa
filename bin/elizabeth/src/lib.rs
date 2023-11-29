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
            inspinia.set_recuperator_fan_speed(speed).await?;
        }
        (DeviceType::Thermostat, ActionType::SetIsEnabled(value)) => {
            inspinia.set_thermostat_enabled(value, action.room).await?;
        }
        (DeviceType::Thermostat, ActionType::SetTemperature(value, relative)) => {
            if relative {
                let current = inspinia
                    .get_thermostat_temperature_in_room(action.room)
                    .await?;

                debug!("current: {}", current);
                debug!("value: {}", value);

                inspinia
                    .set_thermostat_temperature(current + value, action.room)
                    .await?;
            } else {
                inspinia
                    .set_thermostat_temperature(value, action.room)
                    .await?;
            }
        }
        _ => (),
    }

    Ok(())
}
