use std::net::TcpStream;

use codec::Uuid;
use codec::enc::{
    Encode as _,
    EncodeErrorContext as _,
};
use data::packet::{
    Packet,
    ReadPacket as _,
    WritePacket as _,
};

use crate::error::Error;

pub fn process(stream: &mut TcpStream) -> Result<(), Error> {
    handle_status_request(stream)?;
    handle_ping_request(stream)?;
    Ok(())
}

fn handle_status_request(stream: &mut TcpStream) -> Result<(), Error> {
    let packet = stream.read_packet()?;

    if !packet.data.is_empty() {
        return Err(Error::InvalidPacketData {
            context: "status_request packet data should be empty".to_owned(),
        });
    }

    let payload = json::object! {
        version: {
            name: "1.21.8",
            protocol: 772,
        },
        players: {
            max: 69,
            online: 42,
            sample: [
                {
                    id: Uuid::null().to_string(),
                    name: "Player 0",
                },
                {
                    id: Uuid::null().to_string(),
                    name: "Player 1",
                },
                {
                    id: Uuid::null().to_string(),
                    name: "Player 2",
                },
            ],
        },
        description: {
            text: "minecraft-rs server 🦀",
            color: "#d34516",
        },
    };

    let mut data = Vec::new();
    payload
        .encode(&mut data)
        .err_context("Failed to encode status response")?;

    let packet = Packet::new(0x00, data);

    stream.write_packet(&packet)?;

    Ok(())
}

fn handle_ping_request(stream: &mut TcpStream) -> Result<(), Error> {
    let packet = stream.read_packet()?;

    // We can safely send back the decoded packet as it shares the same id and data
    // as the response packet (ID, Long).
    stream.write_packet(&packet)?;

    Ok(())
}
