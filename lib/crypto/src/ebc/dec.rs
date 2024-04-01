use cipher::{
    block_padding::{Pkcs7, UnpadError},
    BlockDecryptMut, KeyInit,
};

use crate::Token;

type Aes128EbcDec = ecb::Decryptor<aes::Aes128>;

pub fn decrypt(data: &mut [u8], key: Token<16>) -> Result<&[u8], UnpadError> {
    Aes128EbcDec::new(&key.into()).decrypt_padded_mut::<Pkcs7>(data)
}
