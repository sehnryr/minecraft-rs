mod decode;
mod error;

pub use codec_macros::Decode;
pub use decode::Decode;
pub use error::{
    DecodeError,
    DecodeErrorContext,
};
