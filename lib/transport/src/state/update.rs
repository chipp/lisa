use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    Elisa(crate::elisa::State),
    Elisheba(crate::elisheba::State),
    Elizabeth(crate::elizabeth::State),
    Isabel(crate::isabel::State),
}
