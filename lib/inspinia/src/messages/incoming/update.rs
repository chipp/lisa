use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageContent {
    pub force: bool,
    pub id: String,
    pub value: String,
}
