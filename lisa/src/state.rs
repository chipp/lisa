use super::DeviceId;
use alice::{Mode, ModeFunction, StateCapability, StateDevice, StateProperty};

use crate::DeviceType::*;
use crate::Room::*;

pub fn state_for_device(device_id: DeviceId) -> Option<StateDevice> {
    let DeviceId { room, device_type } = &device_id;
    match (room, device_type) {
        (Nursery, TemperatureSensor) => Some(StateDevice::new_with_properties(
            device_id.to_string(),
            vec![
                StateProperty::humidity(35.0),
                StateProperty::temperature(22.0),
            ],
        )),
        (Bedroom, TemperatureSensor) => Some(StateDevice::new_with_properties(
            device_id.to_string(),
            vec![
                StateProperty::humidity(45.0),
                StateProperty::temperature(23.0),
            ],
        )),
        (LivingRoom, TemperatureSensor) => Some(StateDevice::new_with_properties(
            device_id.to_string(),
            vec![
                StateProperty::humidity(55.0),
                StateProperty::temperature(24.0),
            ],
        )),
        (Hallway, VacuumCleaner) => Some(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![
                StateCapability::on_off(false),
                StateCapability::mode(ModeFunction::FanSpeed, Mode::Quiet),
            ],
        )),
        (Corridor, VacuumCleaner) => Some(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![
                StateCapability::on_off(false),
                StateCapability::mode(ModeFunction::FanSpeed, Mode::Quiet),
            ],
        )),
        (Bathroom, VacuumCleaner) => Some(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![
                StateCapability::on_off(false),
                StateCapability::mode(ModeFunction::FanSpeed, Mode::Quiet),
            ],
        )),
        (Nursery, VacuumCleaner) => Some(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![
                StateCapability::on_off(false),
                StateCapability::mode(ModeFunction::FanSpeed, Mode::Quiet),
            ],
        )),
        (Bedroom, VacuumCleaner) => Some(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![
                StateCapability::on_off(false),
                StateCapability::mode(ModeFunction::FanSpeed, Mode::Quiet),
            ],
        )),
        (Kitchen, VacuumCleaner) => Some(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![
                StateCapability::on_off(false),
                StateCapability::mode(ModeFunction::FanSpeed, Mode::Quiet),
            ],
        )),
        (LivingRoom, VacuumCleaner) => Some(StateDevice::new_with_capabilities(
            device_id.to_string(),
            vec![
                StateCapability::on_off(false),
                StateCapability::mode(ModeFunction::FanSpeed, Mode::Quiet),
            ],
        )),
        _ => None,
    }
}
