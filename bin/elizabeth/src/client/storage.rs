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

    pub async fn apply_state(&self, state: &State) {
        let mut devices = self.devices.lock().await;
        let device = devices
            .iter_mut()
            .find(|device| device.room == state.room && device.device_type == state.device_type);

        if let Some(device) = device {
            let mut found = false;

            for capability in device.capabilities.iter_mut() {
                match (&capability, &state.capability) {
                    (IsEnabled(..), IsEnabled(..)) => {
                        *capability = state.capability;
                        found = true;
                        break;
                    }
                    (FanSpeed(..), FanSpeed(..)) => {
                        *capability = state.capability;
                        found = true;
                        break;
                    }
                    (CurrentTemperature(..), CurrentTemperature(..)) => {
                        *capability = state.capability;
                        found = true;
                        break;
                    }
                    (Temperature(..), Temperature(..)) => {
                        *capability = state.capability;
                        found = true;
                        break;
                    }
                    _ => (),
                }
            }

            if !found {
                debug!(
                    "{}/{} not found {:?}",
                    state.room, state.device_type, state.capability
                );
                device.capabilities.push(state.capability);
            }
        } else {
            devices.push(Device {
                room: state.room,
                device_type: state.device_type,
                capabilities: vec![state.capability],
            });
        }
    }

    pub async fn get_capabilities(&self, room: Room, device_type: DeviceType) -> Vec<Capability> {
        let devices = self.devices.lock().await;
        let device = devices
            .iter()
            .find(|device| device.room == room && device.device_type == device_type);

        debug!("device: {:?}", device);

        if let Some(device) = device {
            device.capabilities.clone()
        } else {
            vec![]
        }
    }
}

#[derive(Debug)]
struct Device {
    room: Room,
    device_type: DeviceType,
    capabilities: Vec<Capability>,
}
