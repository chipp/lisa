mod vacuum;
use vacuum::{update_vacuum, VacuumUpdate};

mod thermostat;
use thermostat::{update_thermostats, ThermostatUpdate};

mod recuperator;
use recuperator::{update_recuperator, RecuperatorUpdate};

use std::{str::FromStr, sync::Arc};

use crate::{DeviceId, Result};
use crate::{DeviceType::*};

use alice::{
    ModeFunction, RangeFunction, StateCapability, StateUpdateResult, ToggleFunction,
    UpdateStateDevice, UpdateStateErrorCode, UpdatedDeviceState,
};
use elisheba::Command as VacuumCommand;
use tokio::sync::Mutex;

pub async fn update_devices_state<'a, F>(
    devices: Vec<UpdateStateDevice<'a>>,
    send_vacuum_command: Arc<Mutex<impl Fn(VacuumCommand) -> F>>,
) -> Vec<UpdatedDeviceState>
where
    F: std::future::Future<Output = Result<()>>,
{
    let mut vacuum_update = VacuumUpdate::default();
    let mut thermostats_updates = vec![];
    let mut recuperator_update = RecuperatorUpdate::default();

    for device in devices {
        match DeviceId::from_str(device.id) {
            Ok(DeviceId {
                room,
                device_type: VacuumCleaner,
            }) => {
                vacuum_update.rooms.push(room);

                if vacuum_update.is_enabled != None && vacuum_update.work_speed != None {
                    continue;
                }

                for capability in device.capabilities {
                    match capability {
                        StateCapability::OnOff { value } => vacuum_update.is_enabled = Some(value),
                        StateCapability::Mode {
                            function: ModeFunction::WorkSpeed,
                            mode,
                        } => vacuum_update.work_speed = Some(mode),
                        StateCapability::Toggle {
                            function: ToggleFunction::Pause,
                            value,
                        } => vacuum_update.toggle_pause = Some(value),
                        _ => panic!("unsupported capability {:?}", capability),
                    }
                }
            }
            Ok(DeviceId {
                room,
                device_type: Thermostat,
            }) => {
                let mut thermostat_update = ThermostatUpdate {
                    room,
                    is_enabled: None,
                    temperature: None,
                };

                for capability in device.capabilities {
                    match capability {
                        StateCapability::OnOff { value } => {
                            thermostat_update.is_enabled = Some(value)
                        }
                        StateCapability::Range {
                            function: RangeFunction::Temperature,
                            value,
                            relative,
                        } => thermostat_update.temperature = Some((value, relative)),
                        _ => panic!("unsupported capability {:?}", capability),
                    }
                }

                thermostats_updates.push(thermostat_update);
            }
            Ok(DeviceId {
                room: _,
                device_type: Recuperator,
            }) => {
                for capability in device.capabilities {
                    match capability {
                        StateCapability::OnOff { value } => {
                            recuperator_update.is_enabled = Some(value)
                        }
                        StateCapability::Mode {
                            function: ModeFunction::FanSpeed,
                            mode,
                        } => recuperator_update.mode = Some(mode),
                        _ => panic!("unsupported capability {:?}", capability),
                    }
                }
            }
            _ => continue,
        }
    }

    let mut devices = vec![];

    update_vacuum(vacuum_update, &mut devices, send_vacuum_command).await;
    update_thermostats(
        thermostats_updates,
        &mut devices,
    )
    .await;
    update_recuperator(recuperator_update, &mut devices).await;

    devices
}

fn prepare_result(result: &Result<()>) -> StateUpdateResult {
    match result {
        Ok(_) => StateUpdateResult::ok(),
        Err(_) => {
            // TODO:
            StateUpdateResult::error(UpdateStateErrorCode::DeviceUnreachable, String::default())
        }
    }
}
