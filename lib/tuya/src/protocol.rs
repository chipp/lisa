use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crypto::Token;

use crate::error::{Error, Result};

const PREFIX: u32 = 0x0000_55aa;
const SUFFIX: u32 = 0x0000_aa55;
const VERSION_PREFIX_33: &[u8] = b"3.3\0\0\0\0\0\0\0\0\0\0\0\0";

#[derive(Copy, Clone)]
pub enum Command {
    Control = 7,
    Status = 10,
}

pub async fn send_json(
    stream: &mut TcpStream,
    seq: &mut u32,
    command: Command,
    value: &Value,
    key: Token<16>,
) -> Result<Value> {
    *seq += 1;
    let payload = encrypt_payload(value, key)?;
    let packet = pack(*seq, command, &payload);
    stream.write_all(&packet).await?;

    loop {
        let (response_command, response_payload) = read_packet(stream).await?;
        if response_command == command as u32 {
            return decrypt_payload(&response_payload, key);
        }
        // Skip unsolicited packets (e.g. heartbeats) and keep reading.
    }
}

fn encrypt_payload(value: &Value, key: Token<16>) -> Result<Vec<u8>> {
    let mut json = serde_json::to_vec(value)?;
    let encrypted = crypto::ebc::encrypt(&mut json, key)
        .map_err(|_| Error::InvalidPacket)?
        .to_vec();

    let mut payload = VERSION_PREFIX_33.to_vec();
    payload.extend(encrypted);
    Ok(payload)
}

fn decrypt_payload(payload: &[u8], key: Token<16>) -> Result<Value> {
    if payload.is_empty() {
        return Ok(json!({}));
    }

    let payload = payload
        .strip_prefix(VERSION_PREFIX_33)
        .or_else(|| payload.strip_prefix(b"3.3"))
        .unwrap_or(payload);

    let mut data = payload.to_vec();
    let decrypted = crypto::ebc::decrypt(&mut data, key)
        .map_err(|_| Error::InvalidPacket)?
        .to_vec();
    Ok(serde_json::from_slice(&decrypted)?)
}

fn pack(seq: u32, command: Command, payload: &[u8]) -> Vec<u8> {
    let length = payload.len() as u32 + 8;
    let mut packet = Vec::with_capacity(payload.len() + 24);
    packet.extend(PREFIX.to_be_bytes());
    packet.extend(seq.to_be_bytes());
    packet.extend((command as u32).to_be_bytes());
    packet.extend(length.to_be_bytes());
    packet.extend(payload);

    let crc = crc32(&packet);
    packet.extend(crc.to_be_bytes());
    packet.extend(SUFFIX.to_be_bytes());
    packet
}

async fn read_packet(stream: &mut TcpStream) -> Result<(u32, Vec<u8>)> {
    let mut header = [0; 16];
    stream.read_exact(&mut header).await?;

    if u32::from_be_bytes(header[0..4].try_into().unwrap()) != PREFIX {
        return Err(Error::InvalidPacket);
    }

    let command = u32::from_be_bytes(header[8..12].try_into().unwrap());
    let length = u32::from_be_bytes(header[12..16].try_into().unwrap()) as usize;

    // Responses must have at least a 4-byte return code + 4-byte CRC + 4-byte suffix.
    if length < 12 {
        return Err(Error::InvalidPacket);
    }

    let mut rest = vec![0; length];
    stream.read_exact(&mut rest).await?;

    if u32::from_be_bytes(rest[length - 4..length].try_into().unwrap()) != SUFFIX {
        return Err(Error::InvalidPacket);
    }

    // Tuya v3.3 response packets include a 4-byte return code before the payload.
    // Strip it (rest[0..4]) and the trailing CRC + SUFFIX (rest[length-8..length]).
    let payload = rest[4..length - 8].to_vec();
    Ok((command, payload))
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xedb8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn test_pack() {
        let packet = pack(1, Command::Status, b"{}");
        assert_eq!(&packet[0..4], PREFIX.to_be_bytes());
        assert_eq!(&packet[4..8], 1u32.to_be_bytes());
        assert_eq!(&packet[8..12], 10u32.to_be_bytes());
        assert_eq!(&packet[12..16], 10u32.to_be_bytes());
        assert_eq!(&packet[16..18], b"{}");
        assert_eq!(&packet[packet.len() - 4..], SUFFIX.to_be_bytes());
    }
}
