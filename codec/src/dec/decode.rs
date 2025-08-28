use std::io;

use super::error::DecodeError;
use crate::VarInt;
use crate::dec::error::DecodeErrorContext as _;

/// Decode a single value from a reader.
pub trait Decode: Sized {
    /// Decode a value from a reader.
    ///
    /// # Returns
    ///
    /// Returns the decoded value and the number of bytes read from the reader.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError`] if an error occurs while reading from the
    /// reader.
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError>;
}

impl Decode for bool {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        u8::decode(reader).map(|byte| byte != 0)
    }
}

impl Decode for u8 {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = [0; 1];
        reader.read_exact(&mut bytes)?;
        Ok(u8::from_be_bytes(bytes))
    }
}

impl Decode for u16 {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = [0; 2];
        reader.read_exact(&mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }
}

impl Decode for u32 {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = [0; 4];
        reader.read_exact(&mut bytes)?;
        Ok(u32::from_be_bytes(bytes))
    }
}

impl Decode for u64 {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = [0; 8];
        reader.read_exact(&mut bytes)?;
        Ok(u64::from_be_bytes(bytes))
    }
}

impl Decode for f32 {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(f32::from_bits(u32::decode(reader)?))
    }
}

impl Decode for f64 {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(f64::from_bits(u64::decode(reader)?))
    }
}

impl<T> Decode for Option<T>
where
    T: Decode,
{
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut first = [0_u8; 1];
        reader.read_exact(&mut first)?;

        if first[0] == 0 {
            Ok(None)
        } else {
            let mut with_first = io::Read::chain(io::Cursor::new(first), reader);
            let value =
                T::decode(&mut with_first).err_context("Failed to decode optional value")?;
            Ok(Some(value))
        }
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = VarInt::decode(reader)
            .err_context("Failed to decode vec length")?
            .value();

        let len = if len < 0 {
            return Err(DecodeError::InvalidVarInt);
        } else {
            len.cast_unsigned() as usize
        };

        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            let elem = T::decode(reader).err_context("Failed to decode vec element")?;
            vec.push(elem);
        }

        Ok(vec)
    }
}

impl Decode for String {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = VarInt::decode(reader)
            .err_context("Failed to decode string length")?
            .value();

        let len = if len < 0 {
            return Err(DecodeError::InvalidVarInt);
        } else {
            len.cast_unsigned() as usize
        };

        let mut bytes = vec![0; len];
        reader.read_exact(&mut bytes)?;

        Ok(String::from_utf8(bytes)?)
    }
}

macro_rules! impl_tuple_decode {
    ($($T:tt),+ $(,)?) => {
        impl<$($T),*> Decode for ($($T),*,)
        where
            $($T: Decode),+
        {
            fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
                Ok(($($T::decode(reader)?),*,))
            }
        }
    };
}

impl_tuple_decode!(T1);
impl_tuple_decode!(T1, T2);
impl_tuple_decode!(T1, T2, T3);
impl_tuple_decode!(T1, T2, T3, T4);
impl_tuple_decode!(T1, T2, T3, T4, T5);
impl_tuple_decode!(T1, T2, T3, T4, T5, T6);
impl_tuple_decode!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple_decode!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple_decode!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple_decode!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple_decode!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple_decode!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

impl Decode for json::JsonValue {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = VarInt::decode(reader)
            .err_context("Failed to decode json length")?
            .value();

        let len = if len < 0 {
            return Err(DecodeError::InvalidVarInt);
        } else {
            len.cast_unsigned() as usize
        };

        let mut bytes = vec![0; len];
        reader.read_exact(&mut bytes)?;

        let raw_json = str::from_utf8(&bytes)?;
        let json = json::parse(raw_json)?;

        Ok(json)
    }
}

#[allow(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_u8() {
        let mut buffer = [0x01].as_slice();
        let value = u8::decode(&mut buffer).unwrap();
        assert_eq!(value, 1);
    }

    #[test]
    fn decode_u16() {
        let mut buffer = [0x01, 0x02].as_slice();
        let value = u16::decode(&mut buffer).unwrap();
        assert_eq!(value, 258);
    }

    #[test]
    fn decode_u32() {
        let mut buffer = [0x01, 0x02, 0x03, 0x04].as_slice();
        let value = u32::decode(&mut buffer).unwrap();
        assert_eq!(value, 16_909_060);
    }

    #[test]
    fn decode_u64() {
        let mut buffer = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_slice();
        let value = u64::decode(&mut buffer).unwrap();
        assert_eq!(value, 72_623_859_790_382_856);
    }

    #[test]
    fn decode_option() {
        let mut buffer = [0x00].as_slice();
        let value: Option<u8> = Option::decode(&mut buffer).unwrap();
        assert_eq!(value, None);

        let mut buffer = [0x01].as_slice();
        let value: Option<u8> = Option::decode(&mut buffer).unwrap();
        assert_eq!(value, Some(1_u8));
    }

    #[test]
    fn decode_vec() {
        let mut buffer = [0x05, 0x01, 0x02, 0x03, 0x04, 0x05].as_slice();
        let value: Vec<u8> = Vec::decode(&mut buffer).unwrap();
        assert_eq!(value, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn decode_string() {
        let mut buffer = [0x05, b'H', b'e', b'l', b'l', b'o'].as_slice();
        let value = String::decode(&mut buffer).unwrap();
        assert_eq!(value, "Hello");
    }
}
