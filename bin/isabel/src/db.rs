use std::str::FromStr;

use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput},
    Connection, ToSql,
};

use bluetooth::Event;
use transport::Room;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(db_path: &str) -> Self {
        let conn = Connection::open(db_path).expect("valid sqlite path");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS state (
                id INTEGER PRIMARY KEY,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                room TEXT NOT NULL,
                property TEXT NOT NULL,
                value INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();
        Self { conn }
    }

    pub fn save(&self, room: &Room, property: &Event) -> Result<(), rusqlite::Error> {
        let mut insert = self
            .conn
            .prepare("INSERT INTO state (room, property, value) VALUES (?, ?, ?)")
            .unwrap();

        let room = RoomSqlWrapper(*room);

        match property {
            Event::Temperature(value) => {
                insert.execute((&room, "temperature", &value))?;
            }
            Event::Humidity(value) => {
                insert.execute((&room, "humidity", &value))?;
            }
            Event::Battery(value) => {
                insert.execute((&room, "battery", &value))?;
            }
            Event::TemperatureAndHumidity(temp, hum) => {
                insert.execute((&room, "temperature", &temp))?;
                insert.execute((&room, "humidity", &hum))?;
            }
        }

        Ok(())
    }
}

struct RoomSqlWrapper(Room);

impl ToSql for RoomSqlWrapper {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::from(self.0.to_string()))
    }
}

impl FromSql for RoomSqlWrapper {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> FromSqlResult<Self> {
        let value =
            Room::from_str(value.as_str()?).map_err(|e| FromSqlError::Other(Box::new(e)))?;

        Ok(Self(value))
    }
}
