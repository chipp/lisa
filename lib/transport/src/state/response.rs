use crate::{elisa, elisheba, elizabeth, elzhbieta};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Elisa(elisa::State),
    Elisheba(elisheba::State),
    Elzhbieta(elzhbieta::State),
    Elizabeth(elizabeth::CurrentState),
}
