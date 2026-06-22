use std::fmt::Write;

use md5::{Digest, Md5};

use crate::Token;

pub fn digest(data: impl AsRef<[u8]>) -> Token<16> {
    Md5::digest(data.as_ref()).into()
}

pub fn hex(data: impl AsRef<[u8]>) -> String {
    let mut hex = String::with_capacity(32);

    for byte in digest(data) {
        let _ = write!(&mut hex, "{byte:02x}");
    }

    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex as bytes;

    #[test]
    fn test_digest() {
        assert_eq!(digest("hello"), bytes!("5d41402abc4b2a76b9719d911017c592"));
    }

    #[test]
    fn test_hex() {
        assert_eq!(hex("hello"), "5d41402abc4b2a76b9719d911017c592");
    }
}
