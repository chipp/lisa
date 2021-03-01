use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Request<'a> {
    #[serde(borrow = "'a")]
    pub devices: Vec<Device<'a>>,
}

#[derive(Debug, Deserialize)]
pub struct Device<'a> {
    pub id: &'a str,
}
