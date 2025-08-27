use std::io;
use std::net::TcpStream;

use codec::VarInt;
use codec::dec::{
    Decode,
    DecodeError,
    DecodeErrorContext as _,
};
use codec::enc::{
    Encode,
    EncodeError,
    EncodeErrorContext as _,
};

#[derive(Debug)]
pub struct Packet {
    pub id: u32,
    pub data: Vec<u8>,
}

impl Packet {
    #[must_use]
    pub fn new(
        id: u32,
        data: Vec<u8>,
    ) -> Self {
        Packet {
            id,
            data,
        }
    }
}

impl Decode for Packet {
    fn decode<R: io::Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = VarInt::decode(reader)
            .err_context("Failed to decode packet length")?
            .value();

        let mut data = vec![0; len as usize];
        reader.read_exact(&mut data)?;
        let mut data = data.as_slice();

        Ok(Packet {
            id: VarInt::decode(&mut data)?.value(),
            data: data.to_vec(),
        })
    }
}

impl Encode for Packet {
    fn encode<W: io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let id = VarInt::new(self.id);
        let data = &self.data;

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Packet length are max VarInt, so u32 is safe"
        )]
        let packet_len = VarInt::new((id.as_slice().len() + data.len()) as u32);

        let mut written_bytes = 0;

        written_bytes += packet_len.encode(writer)?;
        written_bytes += id.encode(writer)?;

        writer.write_all(data)?;

        Ok(written_bytes + data.len())
    }
}

pub trait ReadPacket {
    /// Reads a packet from the given reader.
    ///
    /// # Returns
    ///
    /// The decoded packet.
    ///
    /// # Errors
    ///
    /// If the packet could not be decoded.
    fn read_packet(&mut self) -> Result<Packet, DecodeError>;
}

impl ReadPacket for TcpStream {
    fn read_packet(&mut self) -> Result<Packet, DecodeError> {
        Packet::decode(self).err_context("Failed to decode packet")
    }
}

pub trait WritePacket {
    /// Writes a packet to the given writer.
    ///
    /// # Returns
    ///
    /// The number of bytes written.
    ///
    /// # Errors
    ///
    /// If the packet could not be encoded.
    fn write_packet(
        &mut self,
        packet: &Packet,
    ) -> Result<usize, EncodeError>;
}

impl WritePacket for TcpStream {
    fn write_packet(
        &mut self,
        packet: &Packet,
    ) -> Result<usize, EncodeError> {
        packet.encode(self).err_context("Failed to encode packet")
    }
}
