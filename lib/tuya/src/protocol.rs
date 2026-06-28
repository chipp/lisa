use log::trace;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crypto::Token;

use crate::error::{Error, Result};

const PREFIX: u32 = 0x0000_55aa;
const SUFFIX: u32 = 0x0000_aa55;
const VERSION_PREFIX_33: &[u8] = b"3.3\0\0\0\0\0\0\0\0\0\0\0\0";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    trace!(
        "tuya send seq={} cmd={} payload={}",
        seq,
        command as u32,
        value
    );
    let payload = encrypt_payload(command, value, key)?;
    let packet = pack(*seq, command, &payload);
    trace!("tuya send packet {} bytes", packet.len());
    stream.write_all(&packet).await?;

    loop {
        let (response_command, response_payload) = read_packet(stream).await?;
        trace!(
            "tuya recv cmd={} payload_len={} bytes={:02x?}",
            response_command,
            response_payload.len(),
            &response_payload[..response_payload.len().min(64)],
        );
        if response_command == command as u32 {
            return decrypt_payload(&response_payload, key);
        }
        trace!(
            "tuya skipping unsolicited packet cmd={} (expected cmd={})",
            response_command,
            command as u32
        );
    }
}

fn encrypt_payload(command: Command, value: &Value, key: Token<16>) -> Result<Vec<u8>> {
    let mut json = serde_json::to_vec(value)?;
    trace!(
        "tuya encrypt plaintext len={} {:?}",
        json.len(),
        std::str::from_utf8(&json).unwrap_or("<non-utf8>")
    );
    let encrypted = crypto::ebc::encrypt(&mut json, key)
        .map_err(|_| Error::InvalidPacket)?
        .to_vec();

    match command {
        Command::Status => {
            // Status (DP_QUERY) does not have a version prefix prepended to the encrypted payload.
            trace!(
                "tuya encrypt Status payload (no version prefix) len={}",
                encrypted.len()
            );
            Ok(encrypted)
        }
        Command::Control => {
            // Control has the version prefix prepended.
            let mut payload = VERSION_PREFIX_33.to_vec();
            payload.extend(encrypted);
            trace!(
                "tuya encrypt Control payload len={} (15-byte prefix + {} encrypted)",
                payload.len(),
                payload.len() - 15
            );
            Ok(payload)
        }
    }
}

fn decrypt_payload(payload: &[u8], key: Token<16>) -> Result<Value> {
    trace!(
        "tuya decrypt raw len={} first_bytes={:02x?}",
        payload.len(),
        &payload[..payload.len().min(20)]
    );

    if payload.is_empty() {
        trace!("tuya decrypt empty payload → returning {{}}");
        return Ok(json!({}));
    }

    let stripped = if let Some(s) = payload.strip_prefix(VERSION_PREFIX_33) {
        trace!("tuya decrypt stripped full 15-byte 3.3 prefix");
        s
    } else if let Some(s) = payload.strip_prefix(b"3.3") {
        trace!("tuya decrypt stripped short 3-byte 3.3 prefix");
        s
    } else {
        trace!("tuya decrypt no version prefix found, using payload as-is");
        payload
    };

    trace!(
        "tuya decrypt after prefix strip len={} first_bytes={:02x?}",
        stripped.len(),
        &stripped[..stripped.len().min(20)]
    );

    let mut data = stripped.to_vec();
    let decrypted = crypto::ebc::decrypt(&mut data, key)
        .map_err(|_| Error::InvalidPacket)?
        .to_vec();

    trace!(
        "tuya decrypt result len={} content={:?}",
        decrypted.len(),
        std::str::from_utf8(&decrypted).unwrap_or("<non-utf8>")
    );

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

    let prefix = u32::from_be_bytes(header[0..4].try_into().unwrap());
    if prefix != PREFIX {
        trace!("tuya read_packet bad prefix {prefix:#010x}");
        return Err(Error::InvalidPacket);
    }

    let seq = u32::from_be_bytes(header[4..8].try_into().unwrap());
    let command = u32::from_be_bytes(header[8..12].try_into().unwrap());
    let length = u32::from_be_bytes(header[12..16].try_into().unwrap()) as usize;
    trace!("tuya read_packet header seq={seq} cmd={command} length={length}");

    // Responses must have at least a 4-byte return code + 4-byte CRC + 4-byte suffix.
    if length < 12 {
        trace!("tuya read_packet length {length} < 12, rejecting");
        return Err(Error::InvalidPacket);
    }

    let mut rest = vec![0; length];
    stream.read_exact(&mut rest).await?;

    let suffix = u32::from_be_bytes(rest[length - 4..length].try_into().unwrap());
    if suffix != SUFFIX {
        trace!("tuya read_packet bad suffix {suffix:#010x}");
        return Err(Error::InvalidPacket);
    }

    let return_code = u32::from_be_bytes(rest[0..4].try_into().unwrap());
    trace!("tuya read_packet return_code={return_code:#010x}");

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
