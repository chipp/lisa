use crate::{elisa, elizabeth};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Elisa(elisa::State),
    Elizabeth(elizabeth::CurrentState),
}
