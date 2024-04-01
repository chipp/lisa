use alice::{StateCapability, StateDevice};
use transport::elisheba::State;
use transport::DeviceId;

pub fn prepare_light_update(state: State) -> StateDevice {
    StateDevice::new_with_capabilities(
        DeviceId::light_at_room(state.room),
        vec![StateCapability::on_off(state.is_enabled)],
    )
}
