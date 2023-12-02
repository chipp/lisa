use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    Elizabeth(crate::elizabeth::State),
    Elisa(crate::elisa::State),
    Isabel(crate::isabel::State),
}
