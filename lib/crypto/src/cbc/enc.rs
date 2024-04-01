use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use cipher::inout::PadError;

use crate::Token;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

const BLOCK_SIZE: usize = 16;

pub fn encrypt(data: &mut Vec<u8>, key: Token<16>, iv: Token<16>) -> Result<&[u8], PadError> {
    let pos = data.len();
    if pos % BLOCK_SIZE != 0 {
        data.append(&mut vec![0; BLOCK_SIZE - pos % BLOCK_SIZE]);
    }

    let ct = Aes128CbcEnc::new(&key.into(), &iv.into()).encrypt_padded_mut::<Pkcs7>(data, pos)?;
    Ok(ct)
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
    fn test_encrypt() {
        let mut data = b"{\"test\":\"message\"}".to_vec();
        let encrypted = encrypt(&mut data, KEY, IV).unwrap();
        assert_eq!(encrypted, ENCRYPTED);
    }
}
