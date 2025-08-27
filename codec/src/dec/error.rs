use alloc::string;
use core::{
    error,
    fmt,
    str,
};
use std::io;

#[derive(Debug)]
pub enum DecodeError {
    Context {
        context: String,
        error: Box<DecodeError>,
    },
    Custom {
        message: String,
    },
    UnexpectedEnd,
    InvalidUtf8(str::Utf8Error),
    InvalidVarInt,
    InvalidVarLong,
}

impl DecodeError {
    #[must_use]
    pub fn context(
        self,
        context: impl Into<String>,
    ) -> Self {
        DecodeError::Context {
            context: context.into(),
            error: Box::new(self),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            DecodeError::Context {
                context,
                error,
            } => write!(f, "{context}: {error}"),
            DecodeError::Custom {
                message,
            } => write!(f, "Custom error: {message}"),
            DecodeError::UnexpectedEnd => write!(f, "Unexpected end of file"),
            DecodeError::InvalidUtf8(err) => write!(f, "Invalid UTF-8 sequence: {err}"),
            DecodeError::InvalidVarInt => write!(f, "Invalid VarInt"),
            DecodeError::InvalidVarLong => write!(f, "Invalid VarLong"),
        }
    }
}

impl error::Error for DecodeError {}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::UnexpectedEof => DecodeError::UnexpectedEnd,
            _ => unreachable!("Unexpected error kind: {:?}", err.kind()),
        }
    }
}

impl From<str::Utf8Error> for DecodeError {
    fn from(err: str::Utf8Error) -> Self { DecodeError::InvalidUtf8(err) }
}

impl From<string::FromUtf8Error> for DecodeError {
    fn from(err: string::FromUtf8Error) -> Self { DecodeError::InvalidUtf8(err.utf8_error()) }
}

pub trait DecodeErrorContext {
    #[must_use]
    fn err_context(
        self,
        context: impl Into<String>,
    ) -> Self;
}

impl<T> DecodeErrorContext for Result<T, DecodeError> {
    fn err_context(
        self,
        context: impl Into<String>,
    ) -> Self {
        self.map_err(|err| err.context(context))
    }
}
