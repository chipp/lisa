mod encryption;
use std::fmt;

pub use encryption::{decrypt, encrypt};
use md5::{Digest, Md5};

use crate::{Result, Token};

#[derive(Debug)]
pub struct Message {
    header: Header,
    checksum: [u8; 16],
    data: Vec<u8>,
}

#[derive(Debug)]
struct InvalidChecksum;

impl fmt::Display for InvalidChecksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid data checksum")
    }
}

impl std::error::Error for InvalidChecksum {}

impl Message {
    pub fn encode(data: Vec<u8>, token: Token<16>, id: u32, send_ts: u32) -> Result<Message> {
        let mut data = data;
        let data = encrypt(&mut data, token)?.to_vec();

        let header = Header {
            id,
            ts: send_ts,
            length: data.len() + 32,
        };

        let checksum = Self::checksum(&header, &token, &data);

        Ok(Message {
            header,
            checksum,
            data,
        })
    }

    pub fn decode(self, token: Token<16>) -> Result<Vec<u8>> {
        let checksum = Self::checksum(&self.header, &token, &self.data);

        if checksum != self.checksum {
            return Err(InvalidChecksum.into());
        }

        let mut data = self.data;
        let mut data = decrypt(&mut data, token)?.to_vec();
        while data.ends_with(&[0x0]) {
            data.pop();
        }

        Ok(data)
    }

    fn checksum(header: &Header, token: &[u8], data: &[u8]) -> [u8; 16] {
        let mut hasher = Md5::new();

        {
            let mut header_data = vec![0; 16];
            header.write_to(&mut header_data);

            hasher.update(header_data);
        }

        hasher.update(token);
        hasher.update(data);

        let mut checksum = [0; 16];
        checksum.copy_from_slice(&hasher.finalize_reset());

        checksum
    }

    pub fn read_from(bytes: &[u8]) -> Message {
        let header = Header::read_from(bytes);

        let mut checksum = [0; 16];
        checksum.copy_from_slice(&bytes[16..32]);

        let mut data = vec![0; header.length - 32];
        data.copy_from_slice(&bytes[32..]);

        Message {
            header,
            checksum,
            data,
        }
    }

    pub fn write_to(self, bytes: &mut [u8]) {
        self.header.write_to(bytes);
        let _ = &bytes[16..32].copy_from_slice(&self.checksum);
        let _ = &bytes[32..].copy_from_slice(&self.data);
    }

    pub fn len(&self) -> usize {
        self.header.length
    }
}

#[derive(Debug)]
pub struct Header {
    pub length: usize,
    pub id: u32,
    pub ts: u32,
}

impl Header {
    pub fn read_from(bytes: &[u8]) -> Header {
        let length = {
            let mut buffer = [0u8; 2];
            buffer.copy_from_slice(&bytes[2..4]);
            u16::from_be_bytes(buffer) as usize
        };

        let id = {
            let mut buffer = [0u8; 4];
            buffer.copy_from_slice(&bytes[8..12]);
            u32::from_be_bytes(buffer)
        };

        let ts = {
            let mut buffer = [0u8; 4];
            buffer.copy_from_slice(&bytes[12..16]);
            u32::from_be_bytes(buffer)
        };

        Header { id, ts, length }
    }

    pub fn write_to(&self, bytes: &mut [u8]) {
        bytes[0] = 0x21;
        bytes[1] = 0x31;

        let length = self.length as u16;
        bytes[2..4].copy_from_slice(&length.to_be_bytes());

        bytes[4..8].fill(0);

        bytes[8..12].copy_from_slice(&self.id.to_be_bytes());
        bytes[12..16].copy_from_slice(&self.ts.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const RESPONSE: [u8; 36] = hex!(
        "2131 0024 0000 0000 2505 2018 5b08 5e5c 0011 2233 4455 6677 8899 aabb ccdd eeff fffe fdfc"
    );

    const HEADER: Header = Header {
        id: 0x25052018,
        ts: 1527275100,
        length: 36,
    };

    const CHECKSUM: [u8; 16] = hex!("00112233445566778899aabbccddeeff");

    #[test]
    fn test_message_read() {
        let message = Message::read_from(&RESPONSE);

        assert_eq!(message.header.length, 36);
        assert_eq!(message.header.id, 0x25052018);
        assert_eq!(message.header.ts, 1527275100);
        assert_eq!(message.checksum, CHECKSUM);
        assert_eq!(message.data, hex!("ff fe fd fc"));
    }

    #[test]
    fn test_message_write() {
        let message = Message {
            header: HEADER,
            checksum: CHECKSUM,
            data: hex!("ff fe fd fc").to_vec(),
        };

        let mut bytes = [0xff; 36];
        message.write_to(&mut bytes);

        assert_eq!(bytes, RESPONSE)
    }

    #[test]
    fn test_message_encode() {
        let json = serde_json::json!({"test": "message"});
        let data = serde_json::to_vec(&json).unwrap();

        let message = Message::encode(data, CHECKSUM, 0x25052018, 1527275101).unwrap();

        assert_eq!(message.header.length, 64);
        assert_eq!(message.header.id, 0x25052018);
        assert_eq!(message.header.ts, 1527275101);

        assert_eq!(
            message.data,
            hex!("22a1 9fb1 3a30 0c7e 932c 52fd 24a2 d430 74ea c69f 3240 0626 5298 3f2f f3e5 53b9")
        );

        assert_eq!(message.checksum, hex!("641404928c1540dc761ee0522bac9eaa"));
    }

    #[test]
    fn test_message_decode() {
        let message = Message {
            header: Header {
                id: 0x25052018,
                ts: 1527275101,
                length: 64,
            },
            checksum: hex!("641404928c1540dc761ee0522bac9eaa"),
            data: hex!(
                "22a1 9fb1 3a30 0c7e 932c 52fd 24a2 d430 74ea c69f 3240 0626 5298 3f2f f3e5 53b9"
            )
            .to_vec(),
        };

        let bytes = message.decode(CHECKSUM).unwrap();
        let json = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap();

        assert_eq!(json, serde_json::json!({"test": "message"}));
    }

    #[test]
    fn test_message_decode_invalid_checksum() {
        let message = Message {
            header: Header {
                id: 0x25052018,
                ts: 1527275101,
                length: 64,
            },
            checksum: hex!("641404928c1540dc761ee0522bac9eab"),
            data: hex!(
                "22a1 9fb1 3a30 0c7e 932c 52fd 24a2 d430 74ea c69f 3240 0626 5298 3f2f f3e5 53b9"
            )
            .to_vec(),
        };

        let error = message.decode(CHECKSUM).unwrap_err();
        error.downcast::<InvalidChecksum>().unwrap();
    }
}
