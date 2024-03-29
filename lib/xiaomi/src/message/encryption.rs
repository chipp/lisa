use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use md5::{Digest, Md5};

use crate::Result;
use crate::Token;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

const BLOCK_SIZE: usize = 16;

pub fn encrypt(data: &mut Vec<u8>, token: Token<16>) -> Result<&[u8]> {
    let (key, iv) = key_iv_from_token(token);

    let pos = data.len();
    if pos % BLOCK_SIZE != 0 {
        data.append(&mut vec![0; BLOCK_SIZE - pos % BLOCK_SIZE]);
    }

    let ct = Aes128CbcEnc::new(&key.into(), &iv.into()).encrypt_padded_mut::<Pkcs7>(data, pos)?;
    Ok(ct)
}

pub fn decrypt(data: &mut [u8], token: Token<16>) -> Result<&[u8]> {
    let (key, iv) = key_iv_from_token(token);

    let pt = Aes128CbcDec::new(&key.into(), &iv.into()).decrypt_padded_mut::<Pkcs7>(data)?;
    Ok(pt)
}

fn key_iv_from_token(token: Token<16>) -> ([u8; 16], [u8; 16]) {
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
    const ENCRYPTED: [u8; 32] =
        hex!("22a1 9fb1 3a30 0c7e 932c 52fd 24a2 d430 74ea c69f 3240 0626 5298 3f2f f3e5 53b9");

    #[test]
    fn test_key_iv() {
        let (key, iv) = key_iv_from_token(TOKEN);

        assert_eq!(key, hex!("6e8311168ee16d6aa1aa48c64145003c"));
        assert_eq!(iv, hex!("6f434fa9acd75da73e5fb999f641cda2"))
    }

    #[test]
    fn test_encrypt() {
        let mut data = b"{\"test\":\"message\"}".to_vec();
        let encrypted = encrypt(&mut data, TOKEN).unwrap();
        assert_eq!(encrypted, ENCRYPTED);
    }

    #[test]
    fn test_decrypt() {
        let mut data = ENCRYPTED.to_vec();
        let message = decrypt(&mut data, TOKEN).unwrap();
        assert_eq!(message, b"{\"test\":\"message\"}");
    }
}
