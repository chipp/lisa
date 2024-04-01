use tokio_util::{
    bytes::Buf,
    codec::{Decoder, Encoder},
};

#[derive(Debug)]
pub struct DnsCoder;

impl Decoder for DnsCoder {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 12 {
            return Ok(None);
        }

        src.reserve(1024);

        let result = src.to_vec();
        src.advance(result.len());

        Ok(Some(result))
    }
}

impl Encoder<Vec<u8>> for DnsCoder {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: Vec<u8>,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        Ok(dst.extend(item))
    }
}
