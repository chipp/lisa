use alice::{StateDevice, StateProperty};
use transport::isabel::State;
use transport::DeviceId;

pub fn prepare_sensor_update(state: State) -> StateDevice {
    let device_id = DeviceId::temperature_sensor_at_room(state.room);
    let mut properties = vec![];

    match state.property {
        transport::isabel::Property::Temperature(temperature) => {
            properties.push(StateProperty::temperature(temperature))
        }
        transport::isabel::Property::Humidity(humidity) => {
            properties.push(StateProperty::humidity(humidity))
        }
        transport::isabel::Property::Battery(battery) => {
            properties.push(StateProperty::battery_level(battery as f32))
        }
        transport::isabel::Property::TemperatureAndHumidity(temperature, humidity) => {
            properties.push(StateProperty::temperature(temperature));
            properties.push(StateProperty::humidity(humidity));
        }
    }

    StateDevice::new_with_properties(device_id, properties)
}
