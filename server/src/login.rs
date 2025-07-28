use std::net::TcpStream;

use codec::Uuid;
use codec::dec::{
    Decode as _,
    DecodeErrorContext as _,
};
use codec::enc::{
    Encode,
    EncodeErrorContext as _,
};
use data::packet::{
    Packet,
    ReadPacket as _,
    WritePacket as _,
};

use crate::error::Error;

pub fn process(stream: &mut TcpStream) -> Result<(), Error> {
    handle_hello(stream)?;
    handle_login_acknowledged(stream)?;
    Ok(())
}

#[derive(Debug, Encode, Clone)]
struct Property {
    name: String,
    value: String,
    #[codec(prefixed_option)]
    signature: Option<String>,
}

fn handle_hello(stream: &mut TcpStream) -> Result<(), Error> {
    let packet = stream.read_packet().err_context("")?;

    let mut data = packet.data.as_slice();

    let player_name = String::decode(&mut data).err_context("Failed to decode player name")?;
    let player_uuid = Uuid::decode(&mut data).err_context("Failed to decode player Uuid")?;

    let properties: Vec<Property> = Vec::new();

    let mut data = Vec::new();
    player_uuid
        .encode(&mut data)
        .err_context("Failed to encode player Uuid")?;
    player_name
        .encode(&mut data)
        .err_context("Failed to encode player name")?;
    properties
        .encode(&mut data)
        .err_context("Failed to encode properties")?;

    let packet = Packet::new(0x02, data);

    stream.write_packet(&packet)?;

    Ok(())
}

fn handle_login_acknowledged(stream: &mut TcpStream) -> Result<(), Error> {
    let packet = stream.read_packet()?;

    if !packet.data.is_empty() {
        return Err(Error::InvalidPacketData {
            context: "login_acknowledged packet data should be empty".to_owned(),
        });
    }

    Ok(())
}
