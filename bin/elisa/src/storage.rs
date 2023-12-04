use log::debug;
use transport::elisa::State;

#[derive(Default)]
pub struct Storage {
    state: Option<State>,
}

impl Storage {
    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn apply_state(&mut self, state: &State) -> bool {
        if self.state.as_ref() != Some(state) {
            debug!("old state: {:?}", self.state);
            debug!("state changed: {:?}", state);

            self.state = Some(state.clone());
            true
        } else {
            false
        }
    }
}
