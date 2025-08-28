use core::{
    error,
    fmt,
};
use std::io;
use std::sync::mpsc;

use codec::dec::DecodeError;
use codec::enc::EncodeError;

#[derive(Debug)]
pub enum Error {
    MpscRecv(mpsc::RecvError),
    TcpStreamClone(io::Error),
    Decode(DecodeError),
    Encode(EncodeError),
    UnknownPacketId(i32),
}

impl fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::MpscRecv(err) => write!(f, "MPSC receive error: {err}"),
            Self::TcpStreamClone(err) => write!(f, "TCP stream clone error: {err}"),
            Self::Decode(err) => write!(f, "Decode error: {err}"),
            Self::Encode(err) => write!(f, "Encode error: {err}"),
            Self::UnknownPacketId(id) => write!(f, "Unknown packet ID: {id}"),
        }
    }
}

impl error::Error for Error {}

impl From<mpsc::RecvError> for Error {
    fn from(err: mpsc::RecvError) -> Self { Self::MpscRecv(err) }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self { Self::Decode(err) }
}

impl From<EncodeError> for Error {
    fn from(err: EncodeError) -> Self { Self::Encode(err) }
}
