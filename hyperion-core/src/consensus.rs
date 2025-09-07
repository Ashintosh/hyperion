use crate::block::block::compute_merkle_root;
use crate::block::{Block, Header, Transaction};
use crate::crypto::{Hashable, HASH_SIZE};

use num_bigint::BigUint;


/// Validate Proof-of-Work for a header
pub fn validate_pow(header: &Header) -> bool {
    let hash = BigUint::from_bytes_be(&header.double_sha256());
    let target = BigUint::from_bytes_be(&header.compact_to_target());
    hash <= target
}

#[cfg(test)]
pub fn fake_validate_pow(hash: [u8; HASH_SIZE], difficulty_compact: u32) -> bool {
    let h = BigUint::from_bytes_be(&hash);
    // fake Header only to call instance method
    let dummy = Header::new(0, 0, difficulty_compact, 0, [0u8; HASH_SIZE], [0u8; HASH_SIZE]);
    let target = BigUint::from_bytes_be(&dummy.compact_to_target());
    h <= target
}

/// Simplified mining: find a nonce that satisfies the target
fn mine(header: &mut Header) {
    let mut nonce = 0;
    while !validate_pow(header) {
        nonce += 1;
        header.nonce = nonce;
    }
}

pub fn mine_block(mut header: Header) -> Header {
    let mut nonce: u64 = 0;
    loop {
        header.nonce = nonce;
        if validate_pow(&header) {
            return header;
        }
        nonce = nonce.wrapping_add(1);  // wrap around if overflow
    }
}

/// Build and mine the genesis block
pub fn create_genesis_block() -> Block {
    let tx = Transaction::new(vec![b"genesis".to_vec()], vec![b"genesis_out".to_vec()])
        .expect("Failed to build genesis tx");

    let merkle_root = compute_merkle_root(&[tx.clone()]);

    let header = Header::new(
        1,             // version
        0,             // timestamp
        0x207fffff,    // easy difficulty
        0,             // nonce will be mined
        [0u8; HASH_SIZE], // prev hash = 0
        merkle_root,
    );

    let mined_header = mine_block(header);
    Block::new(mined_header, vec![tx])
}

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn test_pow_check_fake() {
        let difficulty = 0x207fffff;
        let mut fake_hash = [0u8; HASH_SIZE];
        fake_hash[3] = 1;  // very small number
        assert!(fake_validate_pow(fake_hash, difficulty));
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
}