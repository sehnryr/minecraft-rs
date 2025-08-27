use std::io;

use super::error::EncodeError;
use crate::VarInt;
use crate::enc::error::EncodeErrorContext as _;

/// Encode a single value to a writer.
pub trait Encode: Sized {
    /// Encode a value to a writer.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written to the writer.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] if an error occurs while writing to the
    /// writer.
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError>;
}

impl<T> Encode for &T
where
    T: Encode,
{
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        (**self).encode(writer)
    }
}

impl<T> Encode for &mut T
where
    T: Encode,
{
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        (**self).encode(writer)
    }
}

impl Encode for bool {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        u8::encode(&u8::from(*self), writer)
    }
}

impl Encode for u8 {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(1)
    }
}

impl Encode for u16 {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(2)
    }
}

impl Encode for u32 {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(4)
    }
}

impl Encode for u64 {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(8)
    }
}

impl Encode for f32 {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(4)
    }
}

impl Encode for f64 {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(8)
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        if let Some(value) = self {
            let written_bytes = value
                .encode(writer)
                .err_context("Failed to encode Option::Some(T)")?;
            Ok(written_bytes)
        } else {
            let written_bytes = 0_u8
                .encode(writer)
                .err_context("Failed to encode Option::None")?;
            Ok(written_bytes)
        }
    }
}

impl<T> Encode for &[T]
where
    T: Encode,
{
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "Vec should never have more than u32::MAX elements, so u32 is safe"
        )]
        let mut written_bytes = VarInt::new(self.len() as u32).encode(writer)?;

        for elem in *self {
            written_bytes += elem.encode(writer)?;
        }

        Ok(written_bytes)
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        self.as_slice().encode(writer)
    }
}

impl Encode for &str {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let bytes = self.as_bytes();

        #[allow(
            clippy::cast_possible_truncation,
            reason = "String length should never exceed u32::MAX, so u32 is safe"
        )]
        let written_bytes = VarInt::new(bytes.len() as u32).encode(writer)?;

        writer.write_all(bytes)?;
        Ok(written_bytes + bytes.len())
    }
}

impl Encode for String {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        self.as_str().encode(writer)
    }
}

macro_rules! impl_tuple_encode {
    ($($T:tt),+ $(,)?) => {
        impl<$($T),*> Encode for ($($T),*,)
        where
            $($T: Encode),+
        {
            fn encode<W: io::Write>(
                &self,
                writer: &mut W,
            ) -> Result<usize, EncodeError> {
                let mut written_bytes = 0;

                #[allow(non_snake_case, reason = "reuse type parameter names as tuple bindings")]
                let ($(ref $T),*,) = *self;

                $(
                    written_bytes += Encode::encode($T, writer)?;
                )*

                Ok(written_bytes)
            }
        }
    };
}

impl_tuple_encode!(T1);
impl_tuple_encode!(T1, T2);
impl_tuple_encode!(T1, T2, T3);
impl_tuple_encode!(T1, T2, T3, T4);
impl_tuple_encode!(T1, T2, T3, T4, T5);
impl_tuple_encode!(T1, T2, T3, T4, T5, T6);
impl_tuple_encode!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple_encode!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple_encode!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple_encode!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple_encode!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple_encode!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

#[allow(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_u8() {
        let mut buffer = Vec::new();
        let value = 1_u8;
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
    }

    #[test]
    fn encode_u16() {
        let mut buffer = Vec::new();
        let value = 258_u16;
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01, 0x02]);
    }

    #[test]
    fn encode_u32() {
        let mut buffer = Vec::new();
        let value = 16_909_060_u32;
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn encode_u64() {
        let mut buffer = Vec::new();
        let value = 72_623_859_790_382_856_u64;
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }

    #[test]
    fn encode_option() {
        let mut buffer = Vec::new();
        let value: Option<u8> = None;
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x00]);

        let mut buffer = Vec::new();
        let value: Option<u8> = Some(1_u8);
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
    }

    #[test]
    fn encode_vec() {
        let mut buffer = Vec::new();
        let value: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x05, 0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn encode_string() {
        let mut buffer = Vec::new();
        let value = "Hello".to_owned();
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x05, b'H', b'e', b'l', b'l', b'o']);
    }
}
