use std::{fmt, str::FromStr};

use serde::de::{value, IntoDeserializer};
use serde::{Deserialize, Serialize};

use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};

const BEDROOM_ID: &str = "3cb9f95f-67a6-4554-8b90-57529f190d8e";
const HOME_OFFICE_ID: &str = "0fdf9634-5e47-4ca7-b1eb-3339bbdedc14";
const LIVING_ROOM_ID: &str = "ef8f4a07-6fc4-4b7e-99e2-d1c71f4fd96d";
const NURSERY_ID: &str = "abaff06a-9d8a-49fb-9c20-ba3892f16073";

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Bedroom,
    HomeOffice,
    LivingRoom,
    Nursery,
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

impl FromStr for Room {
    type Err = value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

#[derive(Debug)]
pub struct UnknownRoomId(String);

impl fmt::Display for UnknownRoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Unknown room ID {}", self.0))
    }
}

impl std::error::Error for UnknownRoomId {}

impl FromSql for Room {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        Room::try_from(value.as_str()?).map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

impl<'a> TryFrom<&'a str> for Room {
    type Error = UnknownRoomId;

    fn try_from(id: &'a str) -> Result<Self, UnknownRoomId> {
        match id {
            BEDROOM_ID => Ok(Self::Bedroom),
            HOME_OFFICE_ID => Ok(Self::HomeOffice),
            LIVING_ROOM_ID => Ok(Self::LivingRoom),
            NURSERY_ID => Ok(Self::Nursery),
            _ => Err(UnknownRoomId(id.to_string())),
        }
    }
}

impl ToSql for Room {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        match self {
            Room::Bedroom => Ok(BEDROOM_ID.into()),
            Room::HomeOffice => Ok(HOME_OFFICE_ID.into()),
            Room::LivingRoom => Ok(LIVING_ROOM_ID.into()),
            Room::Nursery => Ok(NURSERY_ID.into()),
        }
    }
}
