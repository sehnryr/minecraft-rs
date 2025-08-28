use std::io;

use super::{
    CONTINUE_MASK,
    SEGMENT_MASK,
};
use crate::dec::{
    Decode,
    DecodeError,
};
use crate::enc::{
    Encode,
    EncodeError,
};

#[derive(Debug, Clone)]
pub struct VarInt(Box<[u8]>);

impl VarInt {
    #[must_use]
    pub fn new(mut value: i32) -> Self {
        let mut bytes = Vec::new();

        loop {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "truncate the value to fit into a byte"
            )]
            let byte = value as u8;
            value >>= 7;

            if value == 0 {
                bytes.push(byte & SEGMENT_MASK);
                break;
            }

            bytes.push(byte & SEGMENT_MASK | CONTINUE_MASK);
        }

        Self(bytes.into_boxed_slice())
    }

    #[must_use]
    pub fn value(&self) -> i32 {
        let mut value = 0;
        let mut shift = 0;

        for byte in &self.0 {
            value |= i32::from(*byte & SEGMENT_MASK) << shift;
            shift += 7;
        }

        value
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] { &self.0 }
}

impl From<VarInt> for i32 {
    fn from(var_int: VarInt) -> Self { var_int.value() }
}

impl Decode for VarInt {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = Vec::new();
        let mut mask = 0xFFFF_FFFF_u32;

        loop {
            let byte = u8::decode(reader)?;

            bytes.push(byte & SEGMENT_MASK);

            #[allow(
                clippy::cast_possible_truncation,
                reason = "truncate the value to fit into a byte"
            )]
            if byte & (!mask as u8) != 0 {
                return Err(DecodeError::InvalidVarInt);
            }

            if byte & CONTINUE_MASK == 0 {
                break;
            }

            mask >>= 7;
        }

        Ok(Self(bytes.into_boxed_slice()))
    }
}

impl Encode for VarInt {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.0.as_ref();
        writer.write_all(bytes)?;
        Ok(bytes.len())
    }
}

#[allow(overflowing_literals, reason = "tests")]
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! var_int {
        ($slice:expr, $expected:expr) => {
            let result = VarInt::decode(&mut $slice.as_slice()).unwrap();
            assert_eq!(result.value(), $expected);
        };
    }

    #[test]
    fn decode_var_int() {
        var_int!([0x00], 0_i32);
        var_int!([0x01], 1_i32);
        var_int!([0x7F], 0x7F_i32);
        var_int!([0x80, 0x01], 0x80_i32);
        var_int!([0xFF, 0xFF, 0x01], 0x7FFF_i32);
        var_int!([0x80, 0x80, 0x02], 0x8000_i32);
        var_int!([0xFF, 0xFF, 0xFF, 0x03], 0x7F_FFFF_i32);
        var_int!([0x80, 0x80, 0x80, 0x04], 0x80_0000_i32);
        var_int!([0xFF, 0xFF, 0xFF, 0xFF, 0x07], 0x7FFF_FFFF_i32);
        var_int!([0x80, 0x80, 0x80, 0x80, 0x08], 0x8000_0000_i32);
        var_int!([0xFF, 0xFF, 0xFF, 0xFF, 0x0F], 0xFFFF_FFFF_i32);

        assert!(matches!(
            VarInt::decode(&mut [0x80, 0x80, 0x80, 0x80, 0x10].as_slice()),
            Err(DecodeError::InvalidVarInt)
        ));
    }
}
