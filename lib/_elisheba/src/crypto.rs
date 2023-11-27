use aes_gcm_siv::{
    aead::{generic_array::GenericArray, Aead, Payload},
    Aes256GcmSiv, KeyInit,
};
use rand::{thread_rng, Rng};

use crate::Result;

pub fn encrypt(data: Vec<u8>, key: [u8; 32]) -> Result<Vec<u8>> {
    let nonce = generate_nonce();
    let mut encrypted = encrypt_with_nonce(data.as_slice().into(), key, &nonce)?;

    let mut combined = Vec::from(nonce);
    combined.append(&mut encrypted);

    Ok(combined)
}

fn encrypt_with_nonce(payload: Payload, key: [u8; 32], nonce: &[u8]) -> Result<Vec<u8>> {
    let key = GenericArray::from_slice(&key);
    let cipher = Aes256GcmSiv::new(key);

    let nonce = GenericArray::from_slice(&nonce);

    let encrypted = cipher.encrypt(nonce, payload)?;
    Ok(encrypted)
}

pub fn decrypt(data: Vec<u8>, key: [u8; 32]) -> Result<Vec<u8>> {
    assert!(data.len() > 12);

    let payload = data.as_slice()[12..].into();
    let nonce = &data.as_slice()[..12];

    decrypt_with_nonce(payload, key, &nonce)
}

fn decrypt_with_nonce(payload: Payload, key: [u8; 32], nonce: &[u8]) -> Result<Vec<u8>> {
    let key = GenericArray::from_slice(&key);
    let cipher = Aes256GcmSiv::new(key);

    let nonce = GenericArray::from_slice(&nonce);

    let decrypted = cipher.decrypt(nonce, payload)?;
    Ok(decrypted)
}

fn generate_nonce() -> [u8; 12] {
    let mut rng = thread_rng();

    let mut nonce = [0; 12];
    rng.fill(&mut nonce[..]);

    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const DECRYPTED: &[u8] = b"{\"test\":true}";
    const ENCRYPTED: &[u8] = &hex!(
        "6762 73b9 87c3 2fd7 34bf 4134 db63 0d2a
         9ac0 bb35 ba8c b5ed 6b5e 4894 a3"
    );

    const KEY: [u8; 32] = hex!(
        "0123 4567 89ab cdef 0123 4567 89ab cdef
         0123 4567 89ab cdef 0123 4567 89ab cdef"
    );
    const NONCE: [u8; 12] = hex!("0123 4567 89ab cdef 0123 4567");

    #[test]
    fn test_encrypt() {
        let encrypted = encrypt_with_nonce(DECRYPTED.into(), KEY, &NONCE).unwrap();
        assert_eq!(encrypted, ENCRYPTED)
    }

    #[test]
    fn test_decrypt() {
        let decrypted = decrypt_with_nonce(ENCRYPTED.into(), KEY, &NONCE).unwrap();
        assert_eq!(decrypted, DECRYPTED)
    }

    #[test]
    fn test_integ() {
        let message: &[u8] = b"The quick brown fox jumps over the lazy dog.";

        let encrypted = encrypt(Vec::from(message), KEY).unwrap();
        let decrypted = decrypt(encrypted, KEY).unwrap();

        assert_eq!(message, decrypted)
    }
}
