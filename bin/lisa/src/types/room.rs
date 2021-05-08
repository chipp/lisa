use std::{fmt, str::FromStr};

use serde::de::{value, IntoDeserializer};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Hallway,
    Corridor,
    Bathroom,
    Nursery,
    Bedroom,
    Kitchen,
    LivingRoom,
    Balcony,
}

impl Room {
    pub const fn all_rooms() -> &'static [Room] {
        use Room::*;

        &[
            Hallway, Corridor, Bathroom, Nursery, Bedroom, Kitchen, LivingRoom, Balcony,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Room::Hallway => "Прихожая",
            Room::Corridor => "Коридор",
            Room::Bathroom => "Ванная",
            Room::Nursery => "Детская",
            Room::Bedroom => "Спальня",
            Room::Kitchen => "Кухня",
            Room::LivingRoom => "Зал",
            Room::Balcony => "Балкон",
        }
    }

    pub fn vacuum_id(&self) -> u8 {
        match self {
            Room::Hallway => 18,
            Room::Corridor => 17,
            Room::Bathroom => 12,
            Room::Nursery => 11,
            Room::Bedroom => 10,
            Room::Kitchen => 16,
            Room::LivingRoom => 15,
            Room::Balcony => 14,
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
