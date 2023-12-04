pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

use transport::{
    isabel::{Property, State},
    Room,
};

#[derive(Clone, Debug, Default, PartialEq)]
struct CurrentState {
    temperature: f32,
    humidity: f32,
    battery: u8,
}

#[derive(Default)]
pub struct Storage {
    bedroom: CurrentState,
    home_office: CurrentState,
    kitchen: CurrentState,
    nursery: CurrentState,
}

impl Storage {
    pub fn apply_update(&mut self, state: &State) -> bool {
        match state.room {
            Room::Bedroom => Self::apply_update_to(&mut self.bedroom, state),
            Room::HomeOffice => Self::apply_update_to(&mut self.home_office, state),
            Room::Kitchen => Self::apply_update_to(&mut self.kitchen, state),
            Room::Nursery => Self::apply_update_to(&mut self.nursery, state),
            _ => false,
        }
    }

    fn apply_update_to(current: &mut CurrentState, state: &State) -> bool {
        let mut changed = false;

        match state.property {
            Property::Temperature(temperature) => {
                if current.temperature != temperature {
                    current.temperature = temperature;
                    changed = true;
                }
            }
            Property::Humidity(humidity) => {
                if current.humidity != humidity {
                    current.humidity = humidity;
                    changed = true;
                }
            }
            Property::Battery(battery) => {
                if current.battery != battery {
                    current.battery = battery;
                    changed = true;
                }
            }
            Property::TemperatureAndHumidity(temperature, humidity) => {
                if current.temperature != temperature {
                    current.temperature = temperature;
                    changed = true;
                }
                if current.humidity != humidity {
                    current.humidity = humidity;
                    changed = true;
                }
            }
        }

        changed
    }
}
