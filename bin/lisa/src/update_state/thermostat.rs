use crate::DeviceId;
use crate::{DeviceType::*, Room};

use log::debug;

use alice::{RangeFunction, UpdateStateCapability, UpdatedDeviceState};

use super::prepare_result;

pub struct ThermostatUpdate {
    pub room: Room,
    pub is_enabled: Option<bool>,
    pub temperature: Option<(f32, bool)>,
}

pub async fn update_thermostats(
    updates: Vec<ThermostatUpdate>,
    devices: &mut Vec<UpdatedDeviceState>,
) {
    debug!("thermostat updates count: {}", updates.len());

    for update in updates {
        let mut capabilities = vec![];

        if let Some(_enabled) = update.is_enabled {
            if let Some(_room) = map_room(&update.room) {
                // let controller = &mut controller.lock().await;

                // // TODO: handle error
                // _ = controller.set_is_enabled_in_room(enabled, room).await;
                capabilities.push(UpdateStateCapability::on_off(prepare_result(&Ok(()))));
            }
        }

        if let Some((_temperature, _relative)) = update.temperature {
            if let Some(_room) = map_room(&update.room) {
                // let controller = &mut controller.lock().await;

                // // TODO: handle error
                // _ = controller
                //     .set_temperature_in_room(temperature, relative, room)
                //     .await;
                capabilities.push(UpdateStateCapability::range(
                    RangeFunction::Temperature,
                    prepare_result(&Ok(())),
                ));
            }
        }

        let device_id = DeviceId {
            room: update.room,
            device_type: Thermostat,
        };

        devices.push(UpdatedDeviceState::new(device_id.to_string(), capabilities));
    }
}

fn map_room(room: &Room) -> Option<inspinia::Room> {
    match room {
        Room::Bedroom => Some(inspinia::Room::Bedroom),
        Room::HomeOffice => Some(inspinia::Room::HomeOffice),
        Room::LivingRoom => Some(inspinia::Room::LivingRoom),
        Room::Nursery => Some(inspinia::Room::Nursery),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_room() {
        assert_eq!(map_room(&Room::Bedroom), Some(inspinia::Room::Bedroom));
        assert_eq!(
            map_room(&Room::HomeOffice),
            Some(inspinia::Room::HomeOffice)
        );
        assert_eq!(
            map_room(&Room::LivingRoom),
            Some(inspinia::Room::LivingRoom)
        );
        assert_eq!(map_room(&Room::Nursery), Some(inspinia::Room::Nursery));
    }
}
