use core::{
    error,
    fmt,
};

use codec::dec::DecodeError;
use codec::enc::EncodeError;

#[derive(Debug)]
pub enum Error {
    Decode(DecodeError),
    Encode(EncodeError),
    InvalidPacketData { context: String },
}

impl fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Decode(err) => write!(f, "Decode error: {err}"),
            Self::Encode(err) => write!(f, "Encode error: {err}"),
            Self::InvalidPacketData {
                context,
            } => write!(f, "Invalid packet data: {context}"),
        }
    }
}

impl error::Error for Error {}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self { Self::Decode(err) }
}

impl From<EncodeError> for Error {
    fn from(err: EncodeError) -> Self { Self::Encode(err) }
}
