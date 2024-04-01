use md5::{Digest, Md5};

use crypto::Token;

pub fn key_iv_from_token(token: Token<16>) -> ([u8; 16], [u8; 16]) {
    let mut hasher = Md5::new();
    hasher.update(token);

    let mut key = [0; 16];
    key.copy_from_slice(&hasher.finalize_reset());

    hasher.update(key);
    hasher.update(token);

    let mut iv = [0; 16];
    iv.copy_from_slice(&hasher.finalize());

    (key, iv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const TOKEN: Token<16> = hex!("00112233445566778899aabbccddeeff");

    #[test]
    fn test_key_iv() {
        let (key, iv) = key_iv_from_token(TOKEN);

        assert_eq!(key, hex!("6e8311168ee16d6aa1aa48c64145003c"));
        assert_eq!(iv, hex!("6f434fa9acd75da73e5fb999f641cda2"))
    }
}
