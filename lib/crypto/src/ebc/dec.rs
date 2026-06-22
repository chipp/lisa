use cipher::block_padding::Error as UnpadError;
use cipher::block_padding::Pkcs7;
use cipher::BlockModeDecrypt;
use cipher::KeyInit;

use crate::Token;

type Aes128EbcDec = ecb::Decryptor<aes::Aes128>;

pub fn decrypt(data: &mut [u8], key: Token<16>) -> Result<&[u8], UnpadError> {
    Aes128EbcDec::new(&key.into()).decrypt_padded::<Pkcs7>(data)
}
