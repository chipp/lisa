use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub devices: Vec<String>,
}
