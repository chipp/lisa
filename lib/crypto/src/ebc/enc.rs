use aes::cipher::{block_padding::Pkcs7, KeyInit};
use cipher::inout::PadError;
use cipher::BlockModeEncrypt;

use crate::Token;

type Aes128EbcEnc = ecb::Encryptor<aes::Aes128>;

const BLOCK_SIZE: usize = 16;

pub fn encrypt(data: &mut Vec<u8>, key: Token<16>) -> Result<&[u8], PadError> {
    let pos = data.len();
    if !pos.is_multiple_of(BLOCK_SIZE) {
        data.append(&mut vec![0; BLOCK_SIZE - pos % BLOCK_SIZE]);
    }

    let ct = Aes128EbcEnc::new(&key.into()).encrypt_padded::<Pkcs7>(data, pos)?;
    Ok(ct)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const KEY: Token<16> = *b"0123456789abcdef";
    const ENCRYPTED: [u8; 32] =
        hex!("ec90e4319a1d62d097c5d67ee451d88c0efea48a0091daa595a3a780fe13bd0e");

    #[test]
    fn test_encrypt() {
        let mut data = b"{\"test\":\"message\"}".to_vec();
        let encrypted = encrypt(&mut data, KEY).unwrap();
        assert_eq!(encrypted, ENCRYPTED);
    }
}
