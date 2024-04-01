use std::{collections::HashMap, net::Ipv4Addr};

use super::Error;

use base64::prelude::*;
use crypto::{cbc::decrypt, Token};
use dns_parser::{rdata::Txt, Name, Packet};
use log::trace;

pub struct ParsedPacket<'p> {
    pub ipv4: Option<Ipv4Addr>,
    pub info: Option<HashMap<String, String>>,
    pub service: Option<String>,
    pub host: Option<Name<'p>>,
    pub port: Option<u16>,
}

pub fn parse_packet<'p>(packet: &Packet<'p>) -> Result<ParsedPacket<'p>, Error> {
    let mut ipv4: Option<Ipv4Addr> = None;
    let mut info: Option<HashMap<String, String>> = None;
    let mut service: Option<String> = None;
    let mut host: Option<Name<'_>> = None;
    let mut ptr: Option<Name<'_>> = None;
    let mut port: Option<u16> = None;

    for answer in packet.answers.iter() {
        if let Some(ptr) = ptr {
            if ptr.to_string() != answer.name.to_string() {
                continue;
            }
        }

        match &answer.data {
            dns_parser::RData::A(data) => {
                host = Some(answer.name);
                ipv4 = Some(data.0);
                trace!("A [{}] {}", answer.name, data.0);
            }
            dns_parser::RData::TXT(data) => {
                let data = parse_txt_record(data);
                trace!("TXT [{}] {data:?}", answer.name);
                info = Some(data);
            }
            dns_parser::RData::PTR(data) => {
                service = Some(answer.name.to_string());
                ptr = Some(data.0);
                trace!("PTR {:?}", data);
            }
            dns_parser::RData::SRV(data) => {
                host = Some(data.target);
                port = Some(data.port);
                trace!("SRV {:?}", data);
            }
            _ => (),
        }
    }

    Ok(ParsedPacket {
        ipv4,
        info,
        service,
        host,
        port,
    })
}

fn parse_txt_record(txt: &Txt<'_>) -> HashMap<String, String> {
    txt.iter()
        .filter_map(|txt| parse_txt_field(txt))
        .collect::<HashMap<_, _>>()
}

fn parse_txt_field(txt: &[u8]) -> Option<(String, String)> {
    let mut parts = txt.splitn(2, |&c| c == b'=');

    let key = String::from_utf8_lossy(parts.next()?).to_string();
    let value = String::from_utf8_lossy(parts.next()?).to_string();

    Some((key, value))
}

pub fn parse_meta(
    info: &mut HashMap<String, String>,
    key: Token<16>,
) -> Result<serde_json::Value, Error> {
    let encrypt = info
        .remove("encrypt")
        .ok_or(Error::MissingInfoField("encrypt"))?;

    let data1 = info
        .remove("data1")
        .ok_or(Error::MissingInfoField("data1"))?;
    let data2 = info.remove("data2");
    let data3 = info.remove("data3");
    let data4 = info.remove("data4");

    let data = [Some(data1), data2, data3, data4]
        .into_iter()
        .filter_map(|x| x)
        .collect::<String>();

    let data = if encrypt == "true" {
        let iv = info.remove("iv").ok_or(Error::MissingInfoField("iv"))?;

        let iv = iv_from_base64(&iv);

        let mut data = BASE64_STANDARD.decode(data).unwrap();
        decrypt(&mut data, key, iv).unwrap().to_vec()
    } else {
        data.into_bytes()
    };

    let meta = serde_json::from_slice(&data)?;
    Ok(meta)
}

fn iv_from_base64(base64: &str) -> [u8; 16] {
    let data = BASE64_STANDARD.decode(base64).unwrap();
    let mut iv = [0; 16];
    iv.copy_from_slice(&data);
    iv
}
