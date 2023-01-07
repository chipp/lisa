use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Debug)]
pub struct UnknownPortType(String);

impl Display for UnknownPortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("Unknown port type {}", self.0))
    }
}

impl Error for UnknownPortType {}

impl FromSql for PortType {
    fn column_result<'a>(value: ValueRef<'a>) -> std::result::Result<Self, FromSqlError> {
        PortType::try_from(value.as_str()?).map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

impl<'a> TryFrom<&'a str> for PortType {
    type Error = UnknownPortType;

    fn try_from(value: &'a str) -> std::result::Result<Self, UnknownPortType> {
        match value {
            "INPUT" => Ok(Self::Input),
            "OUTPUT" => Ok(Self::Output),
            _ => Err(UnknownPortType(value.to_string())),
        }
    }
}
