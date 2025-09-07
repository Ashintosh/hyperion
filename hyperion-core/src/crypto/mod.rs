use crate::block::Serializable;
use sha2::{Digest, Sha256};


pub const HASH_SIZE: usize = 32;

/// Trait for things that can be hashed
pub trait Hashable: Serializable {
    /// Return the double-SHA256 of the serialized representation
    fn double_sha256(&self) -> [u8; HASH_SIZE] {
        // This *shouldn't* fail
        let encoded = self.serialize().expect("Failed to serialize for hashing");
        double_sha256(&encoded)
    }
}

/// Utility function for double SHA-256
pub fn double_sha256(data: &[u8]) -> [u8; HASH_SIZE] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    let mut out = [0u8; HASH_SIZE];
    out.copy_from_slice(&second);
    out
}