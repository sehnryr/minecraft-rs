use codec::dec::Decode;

#[derive(Debug, Decode)]
pub struct Handshake {
    #[codec(varint)]
    pub protocol_version: u32,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
}

#[derive(Debug, Decode, Clone, Copy, PartialEq, Eq)]
#[codec(varint)]
pub enum Intent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}
