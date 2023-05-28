use std::sync::Arc;

use super::prepare_result;
use crate::{DeviceId, DeviceType::*, InspiniaController, Room};

use alice::{Mode, ModeFunction, UpdateStateCapability, UpdatedDeviceState};
use log::debug;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct RecuperatorUpdate {
    pub is_enabled: Option<bool>,
    pub mode: Option<Mode>,
}

pub async fn update_recuperator(
    update: RecuperatorUpdate,
    devices: &mut Vec<UpdatedDeviceState>,
    controller: Arc<Mutex<InspiniaController>>,
) {
    debug!("recuperator is enabled: {:?}", update.is_enabled);
    debug!("recuperator work speed: {:?}", update.mode);

    let mut capabilities = vec![];

    if let Some(enabled) = update.is_enabled {
        let controller = &mut controller.lock().await;

        // TODO: handle error
        _ = controller.set_is_enabled_on_recuperator(enabled).await;
        capabilities.push(UpdateStateCapability::on_off(prepare_result(&Ok(()))));
    }

    if let Some(mode) = update.mode {
        let controller = &mut controller.lock().await;

        // TODO: handle error
        _ = controller
            .set_fan_speed_on_recuperator(map_mode(mode))
            .await;
        capabilities.push(UpdateStateCapability::mode(
            ModeFunction::FanSpeed,
            prepare_result(&Ok(())),
        ));
    }

    let device_id = DeviceId {
        room: Room::LivingRoom,
        device_type: Recuperator,
    };

    devices.push(UpdatedDeviceState::new(device_id.to_string(), capabilities));
}

fn map_mode(mode: Mode) -> alisa::FanSpeed {
    match mode {
        Mode::Quiet => todo!(),
        Mode::Low => alisa::FanSpeed::Low,
        Mode::Normal => todo!(),
        Mode::Medium => alisa::FanSpeed::Medium,
        Mode::High => alisa::FanSpeed::High,
        Mode::Turbo => todo!(),
    }
}
