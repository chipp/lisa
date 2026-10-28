use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    Elisa(crate::elisa::State),
    Elisheba(crate::elisheba::State),
    Elzhbieta(crate::elzhbieta::State),
    Elizabeth(crate::elizabeth::State),
    Elizaveta(crate::elizaveta::State),
    Isabel(crate::isabel::State),
}
