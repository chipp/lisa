use log::debug;
use tokio::sync::Mutex;

use transport::elizabeth::{
    Capability::{self, *},
    State,
};
use transport::{DeviceType, Room};

pub struct Storage {
    devices: Mutex<Vec<Device>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            devices: Mutex::from(vec![]),
        }
    }

    pub async fn apply_state(&self, state: &State) -> bool {
        let mut devices = self.devices.lock().await;
        let device = devices
            .iter_mut()
            .find(|device| device.room == state.room && device.device_type == device.device_type);

        let mut updated = false;

        if let Some(device) = device {
            let mut found = false;

            for capability in device.capabilities.iter_mut() {
                match (&capability, &state.capability) {
                    (IsEnabled(lhs), IsEnabled(rhs)) => {
                        if lhs != rhs {
                            *capability = state.capability.clone();
                            updated = true;
                            break;
                        }
                        found = true;
                    }
                    (FanSpeed(lhs), FanSpeed(rhs)) => {
                        if lhs != rhs {
                            *capability = state.capability.clone();
                            updated = true;
                            break;
                        }
                        found = true;
                    }
                    (CurrentTemperature(lhs), CurrentTemperature(rhs)) => {
                        if lhs != rhs {
                            *capability = state.capability.clone();
                            updated = true;
                            break;
                        }
                        found = true;
                    }
                    (Temperature(lhs), Temperature(rhs)) => {
                        if lhs != rhs {
                            *capability = state.capability.clone();
                            updated = true;
                            break;
                        }
                        found = true;
                    }
                    _ => (),
                }
            }

            if !found {
                device.capabilities.push(state.capability.clone());
                updated = true;
            }
        } else {
            devices.push(Device {
                room: state.room,
                device_type: state.device_type,
                capabilities: vec![state.capability],
            });
            updated = true;
        }

        updated
    }
}

struct Device {
    room: Room,
    device_type: DeviceType,
    capabilities: Vec<Capability>,
}
