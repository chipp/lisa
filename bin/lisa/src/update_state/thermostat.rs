use crate::DeviceId;
use crate::{DeviceType::*, Room};

use log::debug;

use alice::{RangeFunction, UpdateStateCapability, UpdatedDeviceState};

use super::prepare_result;

#[allow(dead_code)]
pub struct ThermostatUpdate {
    pub room: Room,
    pub state: Option<bool>,
    pub temperature: Option<(f32, bool)>,
}

pub fn update_thermostats(updates: Vec<ThermostatUpdate>, devices: &mut Vec<UpdatedDeviceState>) {
    debug!("thermostat updates count: {}", updates.len());

    for update in updates {
        let mut capabilities = vec![];

        if let Some(_enabled) = update.state {
            capabilities.push(UpdateStateCapability::on_off(prepare_result(&Ok(()))));
        }

        if let Some((_temperature, _relative)) = update.temperature {
            capabilities.push(UpdateStateCapability::range(
                RangeFunction::Temperature,
                prepare_result(&Ok(())),
            ));
        }

        let device_id = DeviceId {
            room: update.room,
            device_type: Thermostat,
        };

        devices.push(UpdatedDeviceState::new(device_id.to_string(), capabilities));
    }
}
