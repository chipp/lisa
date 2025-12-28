use aes_gcm::{aead::Aead, aead::KeyInit, Aes256Gcm, Nonce};
use crc32fast::Hasher as Crc32;
use log::debug;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{DecodeError, EncodeError, RpcError};
use crate::{Error, Result};

const SALT: &[u8] = b"TXdfu$jyZ#TZHsg4";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalProtocolVersion {
    L01,
}

impl LocalProtocolVersion {
    pub fn as_bytes(self) -> [u8; 3] {
        match self {
            LocalProtocolVersion::L01 => *b"L01",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MessageProtocol {
    HelloRequest = 0,
    HelloResponse = 1,
    PingRequest = 2,
    PingResponse = 3,
    GeneralRequest = 4,
    GeneralResponse = 5,
    RpcRequest = 101,
    RpcResponse = 102,
}

#[derive(Debug, Clone)]
pub struct RoborockMessage {
    pub version: LocalProtocolVersion,
    pub seq: u32,
    pub random: u32,
    pub timestamp: u32,
    pub protocol: MessageProtocol,
    pub payload: Option<Vec<u8>>,
}

impl RoborockMessage {
    pub fn new(
        version: LocalProtocolVersion,
        protocol: MessageProtocol,
        seq: u32,
        random: u32,
        payload: Option<Vec<u8>>,
    ) -> Self {
        Self {
            version,
            seq,
            random,
            timestamp: unix_timestamp(),
            protocol,
            payload,
        }
    }
}

#[derive(Clone)]
pub struct LocalCodec {
    local_key: String,
    connect_nonce: u32,
    ack_nonce: Option<u32>,
}

impl LocalCodec {
    pub fn new(local_key: String, connect_nonce: u32, ack_nonce: Option<u32>) -> Self {
        Self {
            local_key,
            connect_nonce,
            ack_nonce,
        }
    }

    pub fn with_ack_nonce(&self, ack_nonce: u32) -> Self {
        Self {
            local_key: self.local_key.clone(),
            connect_nonce: self.connect_nonce,
            ack_nonce: Some(ack_nonce),
        }
    }

    pub fn build_message(&self, message: &RoborockMessage) -> Result<Vec<u8>> {
        let encrypted_payload = if let Some(payload) = message.payload.as_ref() {
            Some(encrypt_payload(
                message.version,
                &self.local_key,
                message.timestamp,
                message.seq,
                message.random,
                self.connect_nonce,
                self.ack_nonce,
                payload,
            )?)
        } else {
            None
        };

        let mut message_bytes = Vec::with_capacity(
            17 + encrypted_payload
                .as_ref()
                .map_or(0, |payload| payload.len() + 2)
                + 4,
        );
        message_bytes.extend_from_slice(&message.version.as_bytes());
        message_bytes.extend_from_slice(&message.seq.to_be_bytes());
        message_bytes.extend_from_slice(&message.random.to_be_bytes());
        message_bytes.extend_from_slice(&message.timestamp.to_be_bytes());
        message_bytes.extend_from_slice(&(message.protocol as u16).to_be_bytes());
        if let Some(payload) = encrypted_payload {
            message_bytes.extend_from_slice(&(payload.len() as u16).to_be_bytes());
            message_bytes.extend_from_slice(&payload);
        }
        let crc = crc32(&message_bytes).to_be_bytes();
        message_bytes.extend_from_slice(&crc);

        let mut framed = Vec::with_capacity(4 + message_bytes.len());
        framed.extend_from_slice(&(message_bytes.len() as u32).to_be_bytes());
        framed.extend_from_slice(&message_bytes);
        Ok(framed)
    }

    pub fn decode_messages(&self, buffer: &mut Vec<u8>) -> Result<Vec<RoborockMessage>> {
        let mut messages = Vec::new();

        loop {
            if buffer.len() < 4 {
                break;
            }

            if buffer.len() >= 7 && &buffer[4..7] != b"L01" {
                if let Some((prefix_index, data_index)) = find_l01_prefix(buffer) {
                    if prefix_index > 0 {
                        debug!("roborock resync: skipping {} bytes", prefix_index);
                        buffer.drain(0..prefix_index);
                    } else if data_index > 0 {
                        buffer.drain(0..data_index);
                    }
                    continue;
                }
                if buffer.len() > 2 {
                    buffer.drain(0..buffer.len() - 2);
                }
                break;
            }

            let frame_len = u32::from_be_bytes(buffer[0..4].try_into().unwrap()) as usize;
            if frame_len == 0 {
                buffer.drain(0..4);
                continue;
            }

            if buffer.len() < 4 + frame_len {
                break;
            }

            let frame = buffer[4..4 + frame_len].to_vec();
            buffer.drain(0..4 + frame_len);

            if frame.len() < 17 {
                return Err(DecodeError::FrameTooShort.into());
            }

            let version = parse_version(&frame[0..3])?;
            let seq = u32::from_be_bytes(frame[3..7].try_into().unwrap());
            let random = u32::from_be_bytes(frame[7..11].try_into().unwrap());
            let timestamp = u32::from_be_bytes(frame[11..15].try_into().unwrap());
            let protocol = u16::from_be_bytes(frame[15..17].try_into().unwrap());

            let mut payload = None;
            if frame.len() == 17 {
                // No payload and no CRC.
            } else if frame.len() == 21 {
                // No payload, CRC included.
            } else if frame.len() >= 19 {
                let payload_len = u16::from_be_bytes(frame[17..19].try_into().unwrap()) as usize;
                let message_len = 17 + 2 + payload_len;
                if frame.len() < message_len {
                    return Err(DecodeError::PayloadLengthMismatch.into());
                }

                let crc_present = frame.len() >= message_len + 4;
                if payload_len > 0 {
                    if !crc_present {
                        return Err(DecodeError::PayloadCrcMissing.into());
                    }
                    let expected_crc =
                        u32::from_be_bytes(frame[message_len..message_len + 4].try_into().unwrap());
                    let expected_crc_le =
                        u32::from_le_bytes(frame[message_len..message_len + 4].try_into().unwrap());
                    let computed_crc = crc32(&frame[..message_len]);
                    if expected_crc != computed_crc && expected_crc_le != computed_crc {
                        debug!(
                            "crc mismatch payload: len={}, msg_len={}, expected_be={}, expected_le={}, computed={}, frame={}",
                            frame.len(),
                            message_len,
                            expected_crc,
                            expected_crc_le,
                            computed_crc,
                            hex_bytes(&frame),
                        );
                        return Err(DecodeError::CrcMismatch.into());
                    }
                }

                if payload_len > 0 {
                    let payload_bytes = &frame[19..19 + payload_len];
                    payload = Some(decrypt_payload(
                        version,
                        &self.local_key,
                        timestamp,
                        seq,
                        random,
                        self.connect_nonce,
                        self.ack_nonce,
                        payload_bytes,
                    )?);
                }
            } else {
                return Err(DecodeError::PayloadLengthMissing.into());
            }

            let protocol = match protocol {
                0 => MessageProtocol::HelloRequest,
                1 => MessageProtocol::HelloResponse,
                2 => MessageProtocol::PingRequest,
                3 => MessageProtocol::PingResponse,
                4 => MessageProtocol::GeneralRequest,
                5 => MessageProtocol::GeneralResponse,
                101 => MessageProtocol::RpcRequest,
                102 => MessageProtocol::RpcResponse,
                _ => return Err(DecodeError::UnknownProtocol.into()),
            };

            messages.push(RoborockMessage {
                version,
                seq,
                random,
                timestamp,
                protocol,
                payload,
            });
        }

        Ok(messages)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: u32,
    pub method: String,
    pub params: serde_json::Value,
}

impl RpcRequest {
    pub fn new(id: u32, method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            id,
            method: method.into(),
            params,
        }
    }

    pub fn to_payload(&self) -> Result<Vec<u8>> {
        let inner = serde_json::json!({
            "id": self.id,
            "method": self.method,
            "params": self.params,
        });

        let outer = serde_json::json!({
            "dps": {"101": serde_json::to_string(&inner)?},
            "t": unix_timestamp(),
        });

        Ok(serde_json::to_vec(&outer)?)
    }
}

#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub id: Option<u32>,
    pub result: serde_json::Value,
    pub error: Option<RpcError>,
}

pub fn decode_rpc_response(payload: &[u8]) -> Result<RpcResponse> {
    let payload: serde_json::Value = serde_json::from_slice(payload)?;
    let dps = payload
        .get("dps")
        .and_then(|value| value.as_object())
        .ok_or_else(|| DecodeError::MissingDps)?;
    let data_point = dps
        .get("102")
        .and_then(|value| value.as_str())
        .ok_or_else(|| DecodeError::MissingResponse)?;
    let response: serde_json::Value = serde_json::from_str(data_point)?;

    let id = response
        .get("id")
        .and_then(|value| value.as_u64())
        .map(|id| id as u32);
    let mut error = response.get("error").and_then(|value| {
        value.as_str().map(|value| match value {
            "unknown_method" => RpcError::UnknownMethod,
            _ => RpcError::DeviceError,
        })
    });
    let result_value = response.get("result").cloned();
    let result = match result_value {
        Some(serde_json::Value::String(result)) => {
            if result == "ok" {
                serde_json::json!({})
            } else {
                if error.is_none() {
                    if result == "unknown_method" {
                        error = Some(RpcError::UnknownMethod);
                    } else {
                        error = Some(RpcError::UnexpectedResult);
                    }
                }
                serde_json::json!({})
            }
        }
        Some(serde_json::Value::Object(_))
        | Some(serde_json::Value::Array(_))
        | Some(serde_json::Value::Number(_)) => result_value.unwrap(),
        Some(_) => {
            if error.is_none() {
                error = Some(RpcError::InvalidResultType);
            }
            serde_json::json!({})
        }
        None => {
            if error.is_none() {
                error = Some(RpcError::MissingResult);
            }
            serde_json::json!({})
        }
    };

    Ok(RpcResponse { id, result, error })
}

fn parse_version(bytes: &[u8]) -> Result<LocalProtocolVersion> {
    match bytes {
        b"L01" => Ok(LocalProtocolVersion::L01),
        _ => Err(DecodeError::UnknownVersion.into()),
    }
}

fn unix_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

fn crc32(data: &[u8]) -> u32 {
    let mut hasher = Crc32::new();
    hasher.update(data);
    hasher.finalize()
}

fn hex_bytes(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        use std::fmt::Write;
        let _ = write!(out, "{:02x}", byte);
    }
    out
}

fn find_l01_prefix(buffer: &[u8]) -> Option<(usize, usize)> {
    buffer
        .windows(3)
        .position(|window| window == b"L01")
        .map(|index| {
            if index >= 4 {
                (index - 4, index)
            } else {
                (0, index)
            }
        })
}

fn encode_timestamp(timestamp: u32) -> [u8; 8] {
    let hex = format!("{:08x}", timestamp);
    let indices = [5, 6, 3, 7, 1, 2, 0, 4];
    let mut out = [0u8; 8];
    for (pos, idx) in indices.iter().enumerate() {
        out[pos] = hex.as_bytes()[*idx];
    }
    out
}

fn encrypt_payload(
    version: LocalProtocolVersion,
    local_key: &str,
    timestamp: u32,
    sequence: u32,
    nonce: u32,
    connect_nonce: u32,
    ack_nonce: Option<u32>,
    payload: &[u8],
) -> Result<Vec<u8>> {
    match version {
        LocalProtocolVersion::L01 => encrypt_gcm_l01(
            local_key,
            timestamp,
            sequence,
            nonce,
            connect_nonce,
            ack_nonce,
            payload,
        ),
    }
}

fn decrypt_payload(
    version: LocalProtocolVersion,
    local_key: &str,
    timestamp: u32,
    sequence: u32,
    nonce: u32,
    connect_nonce: u32,
    ack_nonce: Option<u32>,
    payload: &[u8],
) -> Result<Vec<u8>> {
    match version {
        LocalProtocolVersion::L01 => decrypt_gcm_l01(
            local_key,
            timestamp,
            sequence,
            nonce,
            connect_nonce,
            ack_nonce,
            payload,
        ),
    }
}

fn l01_key(local_key: &str, timestamp: u32) -> [u8; 32] {
    let mut data = Vec::with_capacity(8 + local_key.len() + SALT.len());
    data.extend_from_slice(&encode_timestamp(timestamp));
    data.extend_from_slice(local_key.as_bytes());
    data.extend_from_slice(SALT);
    let digest = Sha256::digest(&data);
    digest.into()
}

fn l01_iv(timestamp: u32, nonce: u32, sequence: u32) -> [u8; 12] {
    let mut data = Vec::with_capacity(12);
    data.extend_from_slice(&sequence.to_be_bytes());
    data.extend_from_slice(&nonce.to_be_bytes());
    data.extend_from_slice(&timestamp.to_be_bytes());
    let digest = Sha256::digest(&data);
    let mut out = [0u8; 12];
    out.copy_from_slice(&digest[..12]);
    out
}

fn l01_aad(
    timestamp: u32,
    nonce: u32,
    sequence: u32,
    connect_nonce: u32,
    ack_nonce: Option<u32>,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(20);
    data.extend_from_slice(&sequence.to_be_bytes());
    data.extend_from_slice(&connect_nonce.to_be_bytes());
    if let Some(ack) = ack_nonce {
        data.extend_from_slice(&ack.to_be_bytes());
    }
    data.extend_from_slice(&nonce.to_be_bytes());
    data.extend_from_slice(&timestamp.to_be_bytes());
    data
}

fn encrypt_gcm_l01(
    local_key: &str,
    timestamp: u32,
    sequence: u32,
    nonce: u32,
    connect_nonce: u32,
    ack_nonce: Option<u32>,
    payload: &[u8],
) -> Result<Vec<u8>> {
    let key = l01_key(local_key, timestamp);
    let iv = l01_iv(timestamp, nonce, sequence);
    let aad = l01_aad(timestamp, nonce, sequence, connect_nonce, ack_nonce);

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| Error::CryptoKeyLength)?;
    let nonce = Nonce::from_slice(&iv);
    let encrypted = cipher
        .encrypt(
            nonce,
            aes_gcm::aead::Payload {
                msg: payload,
                aad: &aad,
            },
        )
        .map_err(|_| EncodeError::GcmEncryptFailed)?;
    Ok(encrypted)
}

fn decrypt_gcm_l01(
    local_key: &str,
    timestamp: u32,
    sequence: u32,
    nonce: u32,
    connect_nonce: u32,
    ack_nonce: Option<u32>,
    payload: &[u8],
) -> Result<Vec<u8>> {
    let ack_nonce = ack_nonce.ok_or_else(|| DecodeError::MissingAckNonce)?;
    let key = l01_key(local_key, timestamp);
    let iv = l01_iv(timestamp, nonce, sequence);
    let aad = l01_aad(timestamp, nonce, sequence, connect_nonce, Some(ack_nonce));

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| Error::CryptoKeyLength)?;
    let nonce = Nonce::from_slice(&iv);
    let decrypted = cipher
        .decrypt(
            nonce,
            aes_gcm::aead::Payload {
                msg: payload,
                aad: &aad,
            },
        )
        .map_err(|_| DecodeError::GcmDecryptFailed)?;
    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_timestamp() {
        let encoded = encode_timestamp(0x12345678);
        assert_eq!(&encoded, b"67482315");
    }

    #[test]
    fn test_l01_roundtrip() {
        let codec = LocalCodec::new("0123456789abcdef".to_string(), 12345, Some(22222));
        let message = RoborockMessage {
            version: LocalProtocolVersion::L01,
            seq: 42,
            random: 4242,
            timestamp: 1_700_000_000,
            protocol: MessageProtocol::GeneralRequest,
            payload: Some(b"{\"hello\":1}".to_vec()),
        };

        let frame = codec.build_message(&message).unwrap();
        let mut buffer = frame.clone();
        let decoded = codec.decode_messages(&mut buffer).unwrap();
        assert_eq!(decoded.len(), 1);
        let decoded = &decoded[0];
        assert_eq!(decoded.version, message.version);
        assert_eq!(decoded.seq, message.seq);
        assert_eq!(decoded.random, message.random);
        assert_eq!(decoded.timestamp, message.timestamp);
        assert_eq!(decoded.protocol, message.protocol);
        assert_eq!(decoded.payload.as_deref(), message.payload.as_deref());
    }

    #[test]
    fn test_no_payload_roundtrip() {
        let codec = LocalCodec::new("0123456789abcdef".to_string(), 54321, None);
        let message = RoborockMessage {
            version: LocalProtocolVersion::L01,
            seq: 1,
            random: 54321,
            timestamp: 1_700_000_100,
            protocol: MessageProtocol::HelloRequest,
            payload: None,
        };

        let frame = codec.build_message(&message).unwrap();
        let mut buffer = frame.clone();
        let decoded = codec.decode_messages(&mut buffer).unwrap();
        assert_eq!(decoded.len(), 1);
        let decoded = &decoded[0];
        assert_eq!(decoded.protocol, message.protocol);
        assert_eq!(decoded.payload.as_deref(), None);
    }

    #[test]
    fn test_empty_payload_roundtrip() {
        let codec = LocalCodec::new("0123456789abcdef".to_string(), 54321, Some(11111));
        let message = RoborockMessage {
            version: LocalProtocolVersion::L01,
            seq: 2,
            random: 12345,
            timestamp: 1_700_000_200,
            protocol: MessageProtocol::PingRequest,
            payload: Some(Vec::new()),
        };

        let frame = codec.build_message(&message).unwrap();
        let mut buffer = frame.clone();
        let decoded = codec.decode_messages(&mut buffer).unwrap();
        assert_eq!(decoded.len(), 1);
        let decoded = &decoded[0];
        assert_eq!(decoded.protocol, message.protocol);
        assert_eq!(decoded.payload.as_deref(), Some(&[][..]));
    }

    #[test]
    fn test_resync_on_garbage_prefix() {
        let codec = LocalCodec::new("0123456789abcdef".to_string(), 54321, None);
        let message = RoborockMessage {
            version: LocalProtocolVersion::L01,
            seq: 3,
            random: 22222,
            timestamp: 1_700_000_300,
            protocol: MessageProtocol::HelloRequest,
            payload: None,
        };

        let frame = codec.build_message(&message).unwrap();
        let mut buffer = b"junk".to_vec();
        buffer.extend_from_slice(&frame);
        let decoded = codec.decode_messages(&mut buffer).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].protocol, message.protocol);
    }

    #[test]
    fn test_decode_rpc_response_ok() {
        let payload = serde_json::json!({
            "dps": {
                "102": "{\"id\":123,\"result\":{\"ok\":true}}"
            }
        });

        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let response = decode_rpc_response(&payload_bytes).unwrap();
        assert_eq!(response.id, Some(123));
        assert_eq!(response.error, None);
        assert_eq!(response.result, serde_json::json!({"ok": true}));
    }

    #[test]
    fn test_decode_rpc_response_error() {
        let payload = serde_json::json!({
            "dps": {
                "102": "{\"id\":321,\"error\":\"unknown_method\"}"
            }
        });

        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let response = decode_rpc_response(&payload_bytes).unwrap();
        assert_eq!(response.id, Some(321));
        assert_eq!(response.error, Some(RpcError::UnknownMethod));
    }

    #[test]
    fn test_decode_rpc_response_unknown_method_result() {
        let payload = serde_json::json!({
            "dps": {
                "102": "{\"id\":555,\"result\":\"unknown_method\"}"
            }
        });

        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let response = decode_rpc_response(&payload_bytes).unwrap();
        assert_eq!(response.id, Some(555));
        assert_eq!(response.error, Some(RpcError::UnknownMethod));
        assert_eq!(response.result, serde_json::json!({}));
    }
}
