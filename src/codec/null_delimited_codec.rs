use tokio_util::bytes;
use tokio_util::bytes::BufMut;

pub struct NullDelimitedCodec;

impl Default for NullDelimitedCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl NullDelimitedCodec {
    pub fn new() -> Self {
        NullDelimitedCodec {}
    }
}

impl tokio_util::codec::Decoder for NullDelimitedCodec {
    type Item = bytes::BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(pos) = src.iter().position(|b| *b == 0) {
            let mut line = src.split_to(pos + 1);
            line.truncate(pos); // Remove the null byte
            Ok(Some(line))
        } else {
            Ok(None)
        }
    }
}

impl tokio_util::codec::Encoder<bytes::BytesMut> for NullDelimitedCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: bytes::BytesMut,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        dst.extend_from_slice(&item);
        dst.put_u8(0); // Append null byte
        Ok(())
    }
}
