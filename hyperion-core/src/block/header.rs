use crate::block::Serializable;
use crate::crypto::{HASH_SIZE, Hashable};
use crate::error::header::HeaderError;

use num_bigint::BigUint;
use bincode::{Decode, Encode};


#[derive(Clone, Debug, Encode, Decode)]
pub struct Header {
    pub version: u32,
    pub time: u32,
    pub difficulty_compact: u32,
    pub nonce: u64,
    pub prev_hash: [u8; HASH_SIZE],
    pub merkle_root: [u8; HASH_SIZE],
}

impl Header {
    const EXPONENT_BIAS: u32 = 3;
    const MANTISSA_MASK: u32 = 0x007fffff;

    pub fn new(
        version: u32,
        time: u32,
        difficulty_compact: u32,
        nonce: u64, 
        prev_hash: [u8; HASH_SIZE],
        merkle_root: [u8; HASH_SIZE]
    ) -> Self {
        Self { version, time, difficulty_compact, nonce, prev_hash, merkle_root }
    }

    pub fn validate_pow(&self) -> Result<(), HeaderError> {
        if !crate::consensus::validate_pow(self) {
            return Err(HeaderError::InvalidPoW);
        }
        Ok(())
    }

    /// Convert compact difficulty to 256-bit target
    pub fn compact_to_target(&self) -> [u8; HASH_SIZE] {
        crate::consensus::compact_to_target(self.difficulty_compact)
    }

    
}

impl Serializable for Header {}
impl Hashable for Header {}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Header(hash={:x?}, time={}, nonce={})",
            hex::encode(self.double_sha256()),
            self.time,
            self.nonce,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::consensus;

    use super::*;

    #[test]
    fn test_roundtrip_serialization() {
        let h = Header::new(1, 1234567890, 0x1d00ffff, 42, [0u8; HASH_SIZE], [1u8; HASH_SIZE]);
        let bytes = h.serialize().expect("Failed to serialize header bytes");
        let decoded = Header::from_bytes(&bytes).expect("Failed to decode header from bytes");
        assert_eq!(h.double_sha256(), decoded.double_sha256());
    }

    #[test]
    fn test_pow_check_fake() {
        let difficulty = 0x207fffff;
        let mut fake_hash = [0u8; HASH_SIZE];
        fake_hash[3] = 1;  // very small number
        assert!(consensus::fake_validate_pow(fake_hash, difficulty));
    }

    #[test]
    fn test_serialization_edge_cases() {
        let h = Header::new(u32::MAX, 0, 0x1d00ffff, u64::MAX, [0xFF; HASH_SIZE], [0xAA; HASH_SIZE]);
        let bytes = h.serialize().expect("Failed to serialize header bytes");
        let decoded = Header::from_bytes(&bytes).expect("Failed to decode header from bytes");
        assert_eq!(h.double_sha256(), decoded.double_sha256());
    }

    #[test]
    fn test_pow_failure() {
        let h = Header::new(1, 0, 0x207fffff, 0, [0u8; HASH_SIZE], [0u8; HASH_SIZE]);

        // Artificially create a hash bigger than target
        let fake_hash = [0xFF; HASH_SIZE];

        // Use a dummy Header instance to get target
        let target = Header::new(0, 0, h.difficulty_compact, 0, [0u8; HASH_SIZE], [0u8; HASH_SIZE])
            .compact_to_target();

        let fake_hash_num = BigUint::from_bytes_be(&fake_hash);
        let target_num = BigUint::from_bytes_be(&target);

        // Big hash should be greater than target
        assert!(fake_hash_num > target_num);
    }

    #[test]
    fn test_compact_to_target_known() {
        let bits = 0x1d00ffff; // Bitcoin genesis block
        let header = Header::new(0, 0, bits, 0, [0u8; HASH_SIZE], [0u8; HASH_SIZE]);
        let target = header.compact_to_target();
        let expected_start = [0u8, 0, 0, 0, 255, 255, 0];
        assert_eq!(&target[..7], &expected_start);
    }

    #[test]
    fn test_display() {
        let h = Header::new(1, 123, 0x207fffff, 42, [0u8; HASH_SIZE], [0u8; HASH_SIZE]);
        let s = format!("{}", h);
        assert!(s.contains("hash="));
        assert!(s.contains("time=123"));
        assert!(s.contains("nonce=42"));
    }

    #[test]
    fn test_hash_deterministic() {
        let h = Header::new(1, 123, 0x207fffff, 42, [0u8; HASH_SIZE], [0u8; HASH_SIZE]);
        let h2 = Header::new(1, 123, 0x207fffff, 42, [0u8; HASH_SIZE], [0u8; HASH_SIZE]);
        assert_eq!(h.double_sha256(), h2.double_sha256());
    }

    #[test]
    fn test_difficulty_edges() {
        // "Easy" difficulty → large numeric target
        let easy = Header::new(1, 0, 0x207fffff, 0, [0; HASH_SIZE], [0; HASH_SIZE]);
        // "Hard" difficulty → small numeric target
        let hard = Header::new(1, 0, 0x01000000, 0, [0; HASH_SIZE], [0; HASH_SIZE]);

        let easy_num = BigUint::from_bytes_be(&easy.compact_to_target());
        let hard_num = BigUint::from_bytes_be(&hard.compact_to_target());

        // Easy target must be bigger than hard target
        assert!(easy_num > hard_num);
    }
}