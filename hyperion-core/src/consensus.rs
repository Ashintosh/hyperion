use crate::block::block::compute_merkle_root;
use crate::block::{Block, Header, Transaction};
use crate::chain::Blockchain;
use crate::crypto::{Hashable, HASH_SIZE};

use num_bigint::BigUint;


/// Target block time in seconds
pub const TARGET_BLOCK_TIME: u32 = 600;

/// Difficulty adjustment interval in block
pub const ADJUSTMENT_INTERVAL: usize = 10;

const EXPONENT_BIAS: u32 = 3;
const MANTISSA_MASK: u32 = 0x007fffff;

/// Validate Proof-of-Work for a header
pub fn validate_pow(header: &Header) -> bool {
    let hash = BigUint::from_bytes_be(&header.double_sha256());
    let target = BigUint::from_bytes_be(&header.compact_to_target());
    //print!("Target: {}", target);
    hash <= target
}

pub fn adjust_difficulty(chain: &Blockchain) -> u32 {
    let len = chain.len();
    if len < ADJUSTMENT_INTERVAL || len % ADJUSTMENT_INTERVAL != 0 {
        return chain.latest_block().header.difficulty_compact;
    }

    let first_block = chain.get_block_by_height(len - ADJUSTMENT_INTERVAL).unwrap();
    let last_block = chain.latest_block();

    let actual_time = last_block.header.time.saturating_sub(first_block.header.time);
    let expected_time = TARGET_BLOCK_TIME * ADJUSTMENT_INTERVAL as u32;

    let mut target = BigUint::from_bytes_be(&last_block.header.compact_to_target());
    target *= BigUint::from(actual_time.max(1));
    target /= BigUint::from(expected_time.max(1));

    if target.bits() > 256 {
        target = BigUint::from_bytes_be(&[0xFF; HASH_SIZE]);
    }

    target_to_compact(target)
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
// fn mine(header: &mut Header) {
//     let mut nonce = 0;
//     while !validate_pow(header) {
//         nonce += 1;
//         header.nonce = nonce;
//     }
// }

pub fn mine_block(header: &mut Header) -> Header {
    let mut nonce: u64 = 0;
    loop {
        header.nonce = nonce;
        if validate_pow(&header) {
            return header.clone();
        }
        nonce = nonce.wrapping_add(1);  // wrap around if overflow
    }
}

/// Build and mine the genesis block
pub fn create_genesis_block() -> Block {
    let tx = Transaction::new(vec![b"genesis".to_vec()], vec![b"genesis_out".to_vec()])
        .expect("Failed to build genesis tx");

    let merkle_root = compute_merkle_root(&[tx.clone()]);

    let mut header = Header::new(
        1,             // version
        0,             // timestamp
        0x207fffff,    // easy difficulty
        0,             // nonce will be mined
        [0u8; HASH_SIZE], // prev hash = 0
        merkle_root,
    );

    let mined_header = mine_block(&mut header);
    Block::new(mined_header, vec![tx])
}

/// Convert compact difficulty to 256-bit target
pub fn compact_to_target(difficulty_compact: u32) -> [u8; HASH_SIZE] {
    let exponent = (difficulty_compact >> 24) as u32;
    let mantissa = difficulty_compact & MANTISSA_MASK; // Bitcoin caps highest bit

    let mut target = BigUint::from(mantissa);
    if exponent > EXPONENT_BIAS {
        target <<= 8 * (exponent - EXPONENT_BIAS);
    } else if exponent < EXPONENT_BIAS {
        target >>= 8 * (EXPONENT_BIAS - exponent);
    }

    let bytes = target.to_bytes_be();
    let mut out = [0u8; HASH_SIZE];
    let start = HASH_SIZE - bytes.len();
    out[start..].copy_from_slice(&bytes);
    out
}

/// Convert 256-bit target to compact format
pub fn target_to_compact(target: BigUint) -> u32 {
    let bytes = target.to_bytes_be();
    let mut compact: u32 = 0;

    let mut size = bytes.len() as u32;
    let mut mantissa: u32 = 0;

    if size <= 3 {
        for b in bytes.iter() {
            mantissa = (mantissa << 8) | (*b as u32);
        }
        mantissa <<= 8 * (3 - size);
    } else {
        mantissa = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
    }

    if (mantissa & 0x00800000) != 0 {
        mantissa >>= 8;
        size += 1;
    }

    compact |= size << 24;
    compact |= mantissa & MANTISSA_MASK;

    compact
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