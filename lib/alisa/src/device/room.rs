use std::error::Error;
use std::fmt::Display;

use rusqlite::types::{FromSql, FromSqlError, ValueRef};

const LIVING_ROOM_ID: &str = "ef8f4a07-6fc4-4b7e-99e2-d1c71f4fd96d";
const BEDROOM_ID: &str = "3cb9f95f-67a6-4554-8b90-57529f190d8e";
const NURSERY_ID: &str = "abaff06a-9d8a-49fb-9c20-ba3892f16073";
const HOME_OFFICE_ID: &str = "0fdf9634-5e47-4ca7-b1eb-3339bbdedc14";

#[derive(Debug)]
pub enum Room {
    LivingRoom,
    Bedroom,
    Nursery,
    HomeOffice,
}

#[derive(Debug)]
pub struct UnknownRoomId(String);

impl Display for UnknownRoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("Unknown room ID {}", self.0))
    }
}

impl Error for UnknownRoomId {}

impl FromSql for Room {
    fn column_result<'a>(value: ValueRef<'a>) -> std::result::Result<Self, FromSqlError> {
        Room::try_from(value.as_str()?).map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

impl<'a> TryFrom<&'a str> for Room {
    type Error = UnknownRoomId;

    fn try_from(id: &'a str) -> std::result::Result<Self, UnknownRoomId> {
        match id {
            LIVING_ROOM_ID => Ok(Self::LivingRoom),
            BEDROOM_ID => Ok(Self::Bedroom),
            NURSERY_ID => Ok(Self::Nursery),
            HOME_OFFICE_ID => Ok(Self::HomeOffice),
            _ => Err(UnknownRoomId(id.to_string())),
        }
    }
}
