use log::debug;
use transport::elisa::State;

pub struct Storage {
    state: Option<State>,
}

impl Storage {
    pub fn new() -> Self {
        Self { state: None }
    }

    pub async fn apply_state(&mut self, state: &State) -> bool {
        if self.state != Some(*state) {
            debug!("old state: {:?}", self.state);
            debug!("state changed: {:?}", state);

            self.state = Some(*state);
            true
        } else {
            false
        }
    }
}
