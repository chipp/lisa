use std::{fmt, str::FromStr};

use serde::de::{value, IntoDeserializer};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Bathroom,
    Bedroom,
    Corridor,
    Hallway,
    HomeOffice,
    Kitchen,
    LivingRoom,
    Nursery,
    Toilet,
}

#[allow(dead_code)]
impl Room {
    pub const fn all_rooms() -> &'static [Room] {
        use Room::*;

        &[
            Bathroom, Bedroom, Corridor, Hallway, HomeOffice, Kitchen, LivingRoom, Nursery, Toilet,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Room::Bathroom => "Ванная",
            Room::Bedroom => "Спальня",
            Room::Corridor => "Коридор",
            Room::Hallway => "Прихожая",
            Room::HomeOffice => "Кабинет",
            Room::Kitchen => "Кухня",
            Room::LivingRoom => "Зал",
            Room::Nursery => "Детская",
            Room::Toilet => "Туалет",
        }
    }

    pub fn vacuum_id(&self) -> u8 {
        match self {
            Room::Bathroom => 11,
            Room::Bedroom => 13,
            Room::Corridor => 15,
            Room::Hallway => 12,
            Room::HomeOffice => 17,
            Room::Kitchen => 16,
            Room::LivingRoom => 18,
            Room::Nursery => 14,
            Room::Toilet => 10,
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
