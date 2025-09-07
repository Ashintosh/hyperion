pub mod header;
pub mod transaction;
pub mod block;

pub use header::Header;
pub use transaction::Transaction;
pub use block::Block;

use std::error::Error;
use bincode::{Decode, Encode, decode_from_slice, encode_to_vec, config::standard};


/// Trait for types that can be serialized/deserialize via bincode
pub trait Serializable: Sized + Encode + Decode<()> {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        encode_to_vec(self, standard()).map_err(|e| e.into())
    }

    fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        decode_from_slice(data, standard()).map(|(decoded, _len)| decoded).map_err(|e| e.into())
    }
}
