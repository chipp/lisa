mod vacuum;
use vacuum::{update_vacuum, VacuumUpdate};

mod thermostat;
use thermostat::{update_thermostats, ThermostatUpdate};

use std::{str::FromStr, sync::Arc};

use crate::DeviceType::*;
use crate::{DeviceId, Result};

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

    for device in devices.into_iter() {
        match DeviceId::from_str(device.id) {
            Ok(DeviceId {
                room,
                device_type: VacuumCleaner,
            }) => {
                vacuum_update.rooms.push(room);

                if vacuum_update.state != None && vacuum_update.work_speed != None {
                    continue;
                }

                for capability in device.capabilities.into_iter() {
                    match capability {
                        StateCapability::OnOff { value } => vacuum_update.state = Some(value),
                        StateCapability::Mode {
                            function: ModeFunction::WorkSpeed,
                            mode,
                        } => vacuum_update.work_speed = Some(mode),
                        StateCapability::Toggle {
                            function: ToggleFunction::Pause,
                            value,
                        } => vacuum_update.toggle_pause = Some(value),
                        _ => panic!("unsupported capability"),
                    }
                }
            }
            Ok(DeviceId {
                room,
                device_type: Thermostat,
            }) => {
                let mut thermostat_state = ThermostatUpdate {
                    room,
                    state: None,
                    temperature: None,
                };

                for capability in device.capabilities.into_iter() {
                    match capability {
                        StateCapability::OnOff { value } => thermostat_state.state = Some(value),
                        StateCapability::Range {
                            function: RangeFunction::Temperature,
                            value,
                            relative,
                        } => thermostat_state.temperature = Some((value, relative)),
                        _ => panic!("unsupported capability"),
                    }
                }

                thermostats_updates.push(thermostat_state);
            }
            _ => continue,
        }
    }

    let mut devices = vec![];

    update_vacuum(vacuum_update, &mut devices, send_vacuum_command).await;
    update_thermostats(thermostats_updates, &mut devices);

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
