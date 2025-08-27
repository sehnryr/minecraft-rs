use std::io;

use crate::dec::{
    Decode,
    DecodeError,
    DecodeErrorContext as _,
};
use crate::enc::{
    Encode,
    EncodeError,
    EncodeErrorContext as _,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefixedOption<T> {
    None,
    Some(T),
}

impl<T> PrefixedOption<T> {
    #[must_use]
    pub const fn is_some(&self) -> bool { matches!(*self, Self::Some(_)) }

    #[must_use]
    pub const fn is_none(&self) -> bool { !self.is_some() }

    pub const fn as_ref(&self) -> PrefixedOption<&T> {
        match *self {
            Self::Some(ref value) => PrefixedOption::Some(value),
            Self::None => PrefixedOption::None,
        }
    }

    pub const fn as_mut(&mut self) -> PrefixedOption<&mut T> {
        match *self {
            Self::Some(ref mut value) => PrefixedOption::Some(value),
            Self::None => PrefixedOption::None,
        }
    }
}

impl<T> From<Option<T>> for PrefixedOption<T> {
    fn from(option: Option<T>) -> Self {
        match option {
            Some(value) => Self::Some(value),
            None => Self::None,
        }
    }
}

impl<T> From<PrefixedOption<T>> for Option<T> {
    fn from(option: PrefixedOption<T>) -> Self {
        match option {
            PrefixedOption::Some(value) => Some(value),
            PrefixedOption::None => None,
        }
    }
}

impl<T> Decode for PrefixedOption<T>
where
    T: Decode,
{
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let is_some =
            bool::decode(reader).err_context("Failed to decode prefixed option's boolean")?;

        if is_some {
            let value =
                T::decode(reader).err_context("Failed to decode prefixed option's value")?;
            Ok(Self::Some(value))
        } else {
            Ok(Self::None)
        }
    }
}

impl<T> Encode for PrefixedOption<T>
where
    T: Encode,
{
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        match self {
            Self::Some(value) => {
                let mut written_bytes = true
                    .encode(writer)
                    .err_context("Failed to encode prefixed option's boolean")?;
                written_bytes += value
                    .encode(writer)
                    .err_context("Failed to encode prefixed option's value")?;
                Ok(written_bytes)
            }
            Self::None => {
                let written_bytes = false
                    .encode(writer)
                    .err_context("Failed to encode prefixed option's boolean")?;
                Ok(written_bytes)
            }
        }
    }
}

#[allow(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_prefixed_option() {
        let mut buffer = [0x00].as_slice();
        let value: PrefixedOption<u8> = PrefixedOption::decode(&mut buffer).unwrap();
        assert_eq!(value, PrefixedOption::None);

        let mut buffer = [0x01, 0x01].as_slice();
        let value: PrefixedOption<u8> = PrefixedOption::decode(&mut buffer).unwrap();
        assert_eq!(value, PrefixedOption::Some(1_u8));
    }

    #[test]
    fn encode_prefixed_option() {
        let mut buffer = Vec::new();
        let value: PrefixedOption<u8> = PrefixedOption::None;
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x00]);

        let mut buffer = Vec::new();
        let value: PrefixedOption<u8> = PrefixedOption::Some(1_u8);
        value.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01, 0x01]);
    }
}
