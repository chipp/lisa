mod error;
use error::Error;

use std::collections::HashMap;
use std::path::Path;
use std::result::Result as StdResult;
use std::{error::Error as StdError, hash::Hash};

use crate::{Device, Port, PortName, Properties, Result, Room};

use rusqlite::{params, CachedStatement, Connection};

pub struct DeviceManager {
    connection: Connection,
}

impl DeviceManager {
    pub fn new<P>(db_path: P) -> Result<DeviceManager>
    where
        P: AsRef<Path>,
    {
        Ok(DeviceManager {
            connection: Connection::open(db_path.as_ref())?,
        })
    }

    fn select_controls_in_room(&self) -> CachedStatement<'_> {
        self.connection
            .prepare_cached(
                "SELECT tb_controls.id, tb_control_property.value FROM tb_controls
                INNER JOIN tb_control_property ON tb_controls.id = tb_control_property.control_id
                WHERE tb_controls.page_id = ?
                AND tb_controls.controlName = 'ThermostatPlugin'
                AND tb_control_property.name = 'options'",
            )
            .expect("valid sql statement")
    }

    fn select_ports(&self) -> CachedStatement<'_> {
        self.connection
            .prepare_cached(
                "SELECT tb_ports.id, tb_ports.port_type, tb_port_property.value FROM tb_ports
                INNER JOIN tb_port_property ON tb_ports.id = tb_port_property.port_id
                WHERE control_id = ? AND name = 'name'",
            )
            .expect("valid sql statement")
    }
}

impl DeviceManager {
    pub fn get_thermostat_in_room(&self, room: Room) -> Result<Device> {
        let mut select_controls_in_room = self.select_controls_in_room();
        let controls = select_controls_in_room.query_map([&room], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for control in controls {
            let control = control?;

            let properties: Properties = serde_json::from_str(&control.1)?;
            if !properties.controls.contains(&PortName::Mode) {
                continue;
            }

            let ports = try_collect(self.select_ports().query_map(params![control.0], |r| {
                Ok((
                    r.get(0)?,
                    Port {
                        r#type: r.get(1)?,
                        name: r.get(2)?,
                    },
                ))
            })?)?;

            return Ok(Device {
                id: control.0,
                room,
                properties,
                ports,
            });
        }

        Err(Box::new(Error::NoThermostatInRoom(room)))
    }

    pub fn get_recuperator_in_room(&self, room: Room) -> Result<Device> {
        let mut select_controls_in_room = self.select_controls_in_room();
        let controls = select_controls_in_room.query_map([&room], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for control in controls {
            let control = control?;

            let properties: Properties = serde_json::from_str(&control.1)?;
            if !properties.controls.contains(&PortName::FanSpeed) {
                continue;
            }

            let ports = try_collect(self.select_ports().query_map(params![control.0], |r| {
                Ok((
                    r.get(0)?,
                    Port {
                        r#type: r.get(1)?,
                        name: r.get(2)?,
                    },
                ))
            })?)?;

            return Ok(Device {
                id: control.0,
                room,
                properties,
                ports,
            });
        }

        Err(Box::new(Error::NoRecuperatorInRoom(room)))
    }
}

fn try_collect<K, V, E, I>(iterator: I) -> StdResult<HashMap<K, V>, E>
where
    K: Eq + Hash,
    E: StdError,
    I: Iterator<Item = StdResult<(K, V), E>>,
{
    let mut output = HashMap::new();
    output.reserve(iterator.size_hint().1.unwrap_or_default());

    for tuple in iterator {
        let (key, value) = tuple?;
        output.insert(key, value);
    }

    Ok(output)
}
