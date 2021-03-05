use super::data::Data;

use enumflags2::BitFlags;

use std::fmt;

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum FrameControlFlags {
    IsFactoryNew = 1 << 0,
    IsConnected = 1 << 1,
    IsCentral = 1 << 2,
    IsEncrypted = 1 << 3,
    HasMacAddress = 1 << 4,
    HasCapabilities = 1 << 5,
    HasEvent = 1 << 6,
    HasCustomData = 1 << 7,
    HasSubtitle = 1 << 8,
    HasBinding = 1 << 9,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Temperature(u16),                 // 4100
    Humidity(u16),                    // 4102
    Battery(u8),                      // 4106
    TemperatureAndHumidity(u16, u16), // 4109
}

impl Event {
    fn read_from_data(data: &mut Data) -> Option<Event> {
        let event_type = data.read_u16()?;
        let event_len = data.read_u8()?;

        match (event_type, event_len) {
            (0x1004, 2) => {
                let temperature = data.read_u16()?;
                Some(Event::Temperature(temperature))
            }
            (0x1006, 2) => {
                let humidity = data.read_u16()?;
                Some(Event::Humidity(humidity))
            }
            (0x100a, 1) => {
                let battery = data.read_u8()?;
                Some(Event::Battery(battery))
            }
            (0x100d, 4) => {
                let temperature = data.read_u16()?;
                let humidity = data.read_u16()?;
                Some(Event::TemperatureAndHumidity(temperature, humidity))
            }
            _ => {
                eprintln!("unsupported event ({}, {})", event_type, event_len);
                None
            }
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Temperature(temperature) => write!(f, "T: {}", (*temperature as f32) / 10f32),
            Self::Humidity(humidity) => write!(f, "H: {}", (*humidity as f32) / 10f32),
            Self::Battery(battery) => write!(f, "B: {}", battery),
            Self::TemperatureAndHumidity(temperature, humidity) => write!(
                f,
                "T: {} H: {}",
                (*temperature as f32) / 10f32,
                (*humidity as f32) / 10f32
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MacAddr {
    pub octets: [u8; 6],
}

impl MacAddr {
    fn read_from_data(data: &mut Data) -> Option<MacAddr> {
        let mut octets = [0; 6];
        let bytes = data.read_bytes(6)?;

        octets.copy_from_slice(&bytes);
        octets.reverse();

        Some(MacAddr { octets })
    }

    #[cfg(test)]
    fn from_octets(octets: [u8; 6]) -> MacAddr {
        MacAddr { octets }
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5]
        )
    }
}

pub fn parse_event(bytes: &[u8]) -> Option<(MacAddr, Event)> {
    if bytes.len() < 5 {
        return None;
    }

    let mut data = Data::new(bytes);

    let flags = BitFlags::<FrameControlFlags>::from_bits_truncate(data.read_u16()?);
    data.skip(3);

    let mac_addr = if flags.contains(FrameControlFlags::HasMacAddress) {
        MacAddr::read_from_data(&mut data)
    } else {
        None
    }?;

    let event = if flags.contains(FrameControlFlags::HasEvent) {
        Event::read_from_data(&mut data)
    } else {
        None
    }?;

    Some((mac_addr, event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_humidity_event() {
        let bytes = [
            0x50, 0x20, 0xaa, 0x1, 0x7c, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 0x6, 0x10, 0x2, 0x77,
            0x1,
        ];

        let (mac_addr, event) = parse_event(&bytes).unwrap();
        assert_eq!(
            mac_addr,
            MacAddr::from_octets([0x01, 0x23, 0x45, 0x67, 0x89, 0xab])
        );

        assert_eq!(event, Event::Humidity(375));
    }

    #[test]
    fn test_temperature_event() {
        let bytes = [
            0x50, 0x20, 0xaa, 0x1, 0xa8, 0x45, 0x23, 0x01, 0xef, 0xcd, 0xab, 0x4, 0x10, 0x2, 0xf1,
            0x0,
        ];

        let (mac_addr, event) = parse_event(&bytes).unwrap();
        assert_eq!(
            mac_addr,
            MacAddr::from_octets([0xab, 0xcd, 0xef, 0x01, 0x23, 0x45])
        );

        assert_eq!(event, Event::Temperature(241));
    }

    #[test]
    fn test_battery_event() {
        let bytes = [
            0x50, 0x20, 0xaa, 0x1, 0x81, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xa, 0x10, 0x1, 0x61,
        ];

        let (mac_addr, event) = parse_event(&bytes).unwrap();
        assert_eq!(
            mac_addr,
            MacAddr::from_octets([0x01, 0x01, 0x01, 0x01, 0x01, 0x01])
        );

        assert_eq!(event, Event::Battery(97));
    }

    #[test]
    fn test_temperature_and_humidity_event() {
        let bytes = [
            0x50, 0x20, 0xaa, 0x1, 0xae, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0xd, 0x10, 0x4, 0xf0,
            0x0, 0x6e, 0x1,
        ];

        let (mac_addr, event) = parse_event(&bytes).unwrap();
        assert_eq!(
            mac_addr,
            MacAddr::from_octets([0x11, 0x11, 0x11, 0x11, 0x11, 0x11])
        );

        assert_eq!(event, Event::TemperatureAndHumidity(240, 366));
    }
}
