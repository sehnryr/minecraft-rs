use std::net::TcpStream;

use codec::dec::{
    Decode as _,
    DecodeErrorContext as _,
};
use data::model::Handshake;
use data::packet::ReadPacket as _;

use crate::error::Error;

pub fn process(stream: &mut TcpStream) -> Result<Handshake, Error> {
    let handshake = handle_intention(stream)?;
    Ok(handshake)
}

fn handle_intention(stream: &mut TcpStream) -> Result<Handshake, Error> {
    let packet = stream.read_packet()?;

    let mut data = packet.data.as_slice();

    let handshake = Handshake::decode(&mut data).err_context("Failed to decode handshake")?;

    Ok(handshake)
}
