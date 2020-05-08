use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct RequestPayload {
    pub command: String,
    pub original_utterance: String,
    pub r#type: UtteranceType,
}

#[derive(Deserialize)]
pub enum UtteranceType {
    SimpleUtterance,
    ButtonPressed,
}

#[derive(Deserialize)]
pub struct RequestNlu {
    pub tokens: Vec<String>,
    pub entities: Vec<RequestEntity>,
    pub value: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct RequestEntity {
    pub tokens: EntityTokens,
    pub r#type: EntityType,
}

#[derive(Deserialize)]
pub struct EntityTokens {
    pub start: u8,
    pub end: u8,
}

#[derive(Deserialize)]
pub enum EntityType {
    #[serde(rename = "YANDEX.DATETIME")]
    DateTime,
    #[serde(rename = "YANDEX.FIO")]
    Name,
    #[serde(rename = "YANDEX.GEO")]
    Geo,
    #[serde(rename = "YANDEX.NUMBER")]
    Number,
}
