mod encode;
mod error;

pub use codec_macros::Encode;
pub use encode::Encode;
pub use error::{
    EncodeError,
    EncodeErrorContext,
};
