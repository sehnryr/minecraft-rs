use codec::Uuid;
use codec::dec::Decode;
use codec::enc::Encode;

#[derive(Debug, Decode, Encode)]
pub struct Hello {
    pub name: String,
    pub uuid: Uuid,
}

#[derive(Debug, Decode, Encode)]
pub struct LoginCompression {
    #[codec(varint)]
    pub size: i32,
}
