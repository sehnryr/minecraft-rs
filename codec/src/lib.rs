extern crate alloc;

pub mod dec;
pub mod enc;

const SEGMENT_MASK: u8 = 0b0111_1111;
const CONTINUE_MASK: u8 = 0b1000_0000;

mod prefixed_option;
mod uuid;
mod var_int;
mod var_long;

pub use prefixed_option::PrefixedOption;
pub use uuid::Uuid;
pub use var_int::VarInt;
pub use var_long::VarLong;
