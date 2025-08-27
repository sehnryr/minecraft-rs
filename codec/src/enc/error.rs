use core::{
    error,
    fmt,
};
use std::io;

#[derive(Debug)]
pub enum EncodeError {
    Context {
        context: String,
        error: Box<EncodeError>,
    },
    Custom {
        message: String,
    },
    Io(io::Error),
}

impl EncodeError {
    #[must_use]
    pub fn context(
        self,
        context: impl Into<String>,
    ) -> Self {
        EncodeError::Context {
            context: context.into(),
            error: Box::new(self),
        }
    }

    #[must_use]
    pub fn get_io_error(&self) -> Option<&io::Error> {
        match self {
            EncodeError::Context {
                error, ..
            } => error.get_io_error(),
            EncodeError::Custom {
                ..
            } => None,
            EncodeError::Io(err) => Some(err),
        }
    }
}

impl fmt::Display for EncodeError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            EncodeError::Context {
                context,
                error,
            } => write!(f, "{context}: {error}"),
            EncodeError::Custom {
                message,
            } => write!(f, "Custom error: {message}"),
            EncodeError::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl error::Error for EncodeError {}

impl From<io::Error> for EncodeError {
    fn from(err: io::Error) -> Self { EncodeError::Io(err) }
}

pub trait EncodeErrorContext {
    #[must_use]
    fn err_context(
        self,
        context: impl Into<String>,
    ) -> Self;
}

impl<T> EncodeErrorContext for Result<T, EncodeError> {
    fn err_context(
        self,
        context: impl Into<String>,
    ) -> Self {
        self.map_err(|err| err.context(context))
    }
}
