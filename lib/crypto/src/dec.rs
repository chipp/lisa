use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use cipher::block_padding::UnpadError;

use crate::Token;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub fn decrypt(data: &mut [u8], key: Token<16>, iv: Token<16>) -> Result<&[u8], UnpadError> {
    Aes128CbcDec::new(&key.into(), &iv.into()).decrypt_padded_mut::<Pkcs7>(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const KEY: Token<16> = hex!("6e8311168ee16d6aa1aa48c64145003c");
    const IV: Token<16> = hex!("6f434fa9acd75da73e5fb999f641cda2");
    const ENCRYPTED: [u8; 32] =
        hex!("22a1 9fb1 3a30 0c7e 932c 52fd 24a2 d430 74ea c69f 3240 0626 5298 3f2f f3e5 53b9");

    #[test]
    fn test_decrypt() {
        let mut data = ENCRYPTED.to_vec();
        let decrypted = decrypt(&mut data, KEY, IV).unwrap();
        assert_eq!(decrypted, b"{\"test\":\"message\"}");
    }
}
