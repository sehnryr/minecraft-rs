use std::io::{
    self,
    Read as _,
    Write as _,
};
use std::net::TcpStream;

use codec::VarInt;
use codec::dec::{
    Decode as _,
    DecodeError,
};
use codec::enc::{
    Encode as _,
    EncodeError,
};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

fn parse_len<R: io::Read>(reader: &mut R) -> Result<usize, DecodeError> {
    let len = VarInt::decode(reader)?.value();

    let len = if len < 0 {
        return Err(DecodeError::InvalidVarInt);
    } else {
        len.cast_unsigned() as usize
    };

    Ok(len)
}

#[derive(Debug)]
pub struct Packet {
    pub id: i32,
    pub data: Box<[u8]>,
}

impl Packet {
    #[must_use]
    pub fn new(
        id: i32,
        data: &[u8],
    ) -> Self {
        Packet {
            id,
            data: Box::from(data),
        }
    }

    fn read_packet_uncompressed(from: &mut TcpStream) -> Result<Packet, DecodeError> {
        let packet_len = parse_len(from)?;

        let mut data = vec![0; packet_len];
        from.read_exact(&mut data)?;
        let mut data = data.as_slice();

        Ok(Packet {
            id: VarInt::decode(&mut data)?.value(),
            data: Box::from(data),
        })
    }

    fn read_packet_compressed(from: &mut TcpStream) -> Result<Packet, DecodeError> {
        let packet_len = parse_len(from)?;

        let mut packet_buf = vec![0; packet_len];
        from.read_exact(&mut packet_buf)?;
        let mut packet_buf = packet_buf.as_slice();

        let data_len = parse_len(&mut packet_buf)?;

        if data_len != 0 {
            let mut decoder = ZlibDecoder::new(packet_buf);

            let mut data_buf = vec![0; data_len];
            decoder.read_exact(&mut data_buf)?;
            let mut data_buf = data_buf.as_slice();

            Ok(Packet {
                id: VarInt::decode(&mut data_buf)?.value(),
                data: Box::from(data_buf),
            })
        } else {
            Ok(Packet {
                id: VarInt::decode(&mut packet_buf)?.value(),
                data: Box::from(packet_buf),
            })
        }
    }

    fn write_packet_uncompressed(
        &self,
        to: &mut TcpStream,
    ) -> Result<usize, EncodeError> {
        let id = VarInt::new(self.id);
        let data = self.data.as_ref();

        let packet_len = id.as_slice().len() + data.len();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            reason = "Packet length are max i32::MAX"
        )]
        let packet_len_ = VarInt::new(packet_len as i32);

        packet_len_.encode(to)?;
        id.encode(to)?;

        // use write_all instead of encode to not use &[T]::encode which loops over each
        // element and calls encode on each element
        to.write_all(data)?;

        Ok(packet_len_.as_slice().len() + packet_len)
    }

    fn write_packet_compressed(
        &self,
        to: &mut TcpStream,
        min_compression: usize,
    ) -> Result<usize, EncodeError> {
        let id = VarInt::new(self.id);
        let data = self.data.as_ref();

        let data_len = id.as_slice().len() + data.len();

        if data_len >= min_compression {
            let compressed_data_buf = Vec::with_capacity(data_len);
            let mut encoder = ZlibEncoder::new(compressed_data_buf, Compression::default());

            encoder.write_all(id.as_slice())?;
            encoder.write_all(data)?;

            let compressed_data_buf = encoder.finish()?;
            let compressed_data_len = compressed_data_buf.len();

            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_possible_wrap,
                reason = "Data length are max i32::MAX"
            )]
            let data_len_ = VarInt::new(data_len as i32);

            let packet_len = data_len_.as_slice().len() + compressed_data_len;
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_possible_wrap,
                reason = "Packet length are max i32::MAX"
            )]
            let packet_len_ = VarInt::new(packet_len as i32);

            packet_len_.encode(to)?;
            data_len_.encode(to)?;

            // use write_all instead of encode to not use &[T]::encode which loops over each
            // element and calls encode on each element
            to.write_all(&compressed_data_buf)?;

            Ok(packet_len_.as_slice().len() + packet_len)
        } else {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_possible_wrap,
                reason = "Packet length are max i32::MAX"
            )]
            let packet_len_ = VarInt::new(1 + data_len as i32);

            packet_len_.encode(to)?;
            0_u8.encode(to)?;
            id.encode(to)?;

            // use write_all instead of encode to not use &[T]::encode which loops over each
            // element and calls encode on each element
            to.write_all(data)?;

            Ok(packet_len_.as_slice().len() + 1 + data_len)
        }
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
    fn read_packet(
        &mut self,
        is_compressed: bool,
    ) -> Result<Packet, DecodeError>;
}

impl ReadPacket for TcpStream {
    fn read_packet(
        &mut self,
        is_compressed: bool,
    ) -> Result<Packet, DecodeError> {
        if is_compressed {
            Packet::read_packet_compressed(self)
        } else {
            Packet::read_packet_uncompressed(self)
        }
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
        min_compression: Option<usize>,
    ) -> Result<usize, EncodeError>;
}

impl WritePacket for TcpStream {
    fn write_packet(
        &mut self,
        packet: &Packet,
        min_compression: Option<usize>,
    ) -> Result<usize, EncodeError> {
        if let Some(min_compression) = min_compression {
            packet.write_packet_compressed(self, min_compression)
        } else {
            packet.write_packet_uncompressed(self)
        }
    }
}
