use alloc::fmt;
use std::io;

use crate::dec::{
    Decode,
    DecodeError,
};
use crate::enc::{
    Encode,
    EncodeError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uuid(u128);

impl Uuid {
    #[must_use]
    pub const fn null() -> Self { Self(0) }
}

impl fmt::Display for Uuid {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.0 >> 24,
            self.0 >> 20,
            self.0 >> 16,
            self.0 >> 12,
            self.0,
        )
    }
}

impl Decode for Uuid {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = [0; 16];
        reader.read_exact(&mut bytes)?;
        Ok(Self(u128::from_be_bytes(bytes)))
    }
}

impl Encode for Uuid {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.0.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(16)
    }
}

#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::unusual_byte_groupings, reason = "uuid grouping")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_null() {
        assert_eq!(
            "00000000-0000-0000-0000-000000000000".to_owned(),
            format!("{}", Uuid::null())
        );
    }

    #[test]
    fn decode_uuid() {
        #[rustfmt::skip]
        let mut buffer = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
        ]
        .as_slice();
        let value = Uuid::decode(&mut buffer).unwrap();
        assert_eq!(value, Uuid(0x10203040_5060_7080_90A0_B0C0D0E0F10_u128));
    }

    #[test]
    fn encode_uuid() {
        let mut buffer = Vec::new();
        let value = Uuid(0x10203040_5060_7080_90A0_B0C0D0E0F10_u128);
        value.encode(&mut buffer).unwrap();
        #[rustfmt::skip]
        assert_eq!(buffer, vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
        ]);
    }
}
