use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Debug)]
pub struct UnknownPortType(String);

impl fmt::Display for UnknownPortType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Unknown port type {}", self.0))
    }
}

impl Error for UnknownPortType {}

impl FromSql for PortType {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        PortType::try_from(value.as_str()?).map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

impl TryFrom<&str> for PortType {
    type Error = UnknownPortType;

    fn try_from(value: &str) -> Result<Self, UnknownPortType> {
        match value {
            "INPUT" => Ok(Self::Input),
            "OUTPUT" => Ok(Self::Output),
            _ => Err(UnknownPortType(value.to_string())),
        }
    }
}
