use std::{fmt, str::FromStr};

use serde::de::{value, IntoDeserializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Hallway,
    Corridor,
    Bathroom,
    Nursery,
    Bedroom,
    Kitchen,
    LivingRoom,
}

impl Room {
    pub fn name(&self) -> &str {
        match self {
            Room::Hallway => "Прихожая",
            Room::Corridor => "Коридор",
            Room::Bathroom => "Ванная",
            Room::Nursery => "Детская",
            Room::Bedroom => "Спальня",
            Room::Kitchen => "Кухня",
            Room::LivingRoom => "Зал",
        }
    }
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
