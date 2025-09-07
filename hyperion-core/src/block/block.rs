use crate::block::{Header, Serializable, Transaction};
use crate::crypto::{HASH_SIZE, Hashable, double_sha256};

use bincode::{Decode, Encode};


/// A block contains a header and a list of transactions.
#[derive(Clone, Encode, Decode)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub const MERKLE_PAIR_SIZE: usize = HASH_SIZE * 2;

    pub fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self { header, transactions }
    }

    /// Validate block (simplified)
    /// - PoW is valid
    /// - Merkle root matches tx list (stub for now)
    fn validate(&self) -> bool {
        self.header.validate_pow() && self.validate_merkle_root()
    }

    /// Compute merkle root of transactions
    pub fn validate_merkle_root(&self) -> bool {
        let merkle = compute_merkle_root(&self.transactions);
        merkle == self.header.merkle_root
    }

    #[cfg(test)]
    fn new_with_merkle(header: Header, txs: Vec<Transaction>) -> Self {
        let mut block = Self::new(header, txs);
        let merkle = compute_merkle_root(&block.transactions);
        block.header.merkle_root = merkle;
        block
    }
}

impl Serializable for Block {}
impl Hashable for Block {}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block(hash={:x?}, txs={}, header={})",
            hex::encode(self.double_sha256()),
            self.transactions.len(),
            self.header,
        )
    }
}

pub fn compute_merkle_root(transactions: &[Transaction]) -> [u8; HASH_SIZE] {
    if transactions.is_empty() {
        return [0u8; HASH_SIZE];
    }

    let mut layer: Vec<[u8; HASH_SIZE]> = transactions.iter().map(|tx| tx.double_sha256()).collect();

    while layer.len() > 1 {
        let mut next_layer = vec![];

        for i in (0..layer.len()).step_by(2) {
            let left = layer[i];
            let right = if i + 1 < layer.len() {
                layer[i + 1]
            } else {
                layer[i]  // duplicate last if odd
            };

            let mut combined = Vec::with_capacity(Block::MERKLE_PAIR_SIZE);
            combined.extend_from_slice(&left);
            combined.extend_from_slice(&right);
            next_layer.push(double_sha256(&combined));
        }

        layer = next_layer;
    }

    layer[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{self, Header};

    #[test]
    fn test_block_roundtrip_serialization() {
        // create two transactions
        let tx1 = Transaction::new(vec![b"in1".to_vec()], vec![b"out1".to_vec()]);
        let tx2 = Transaction::new(vec![b"in2".to_vec()], vec![b"out2".to_vec()]);

        let header = Header::new(1, 1234567890, 0x1d00ffff, 42, [0u8; HASH_SIZE], [0u8; 32]);

        // create block, automatically computing merkle root
        let block = Block::new_with_merkle(header, vec![tx1.clone(), tx2.clone()]);

        // serialize & deserialize via Serializable trait
        let bytes = block.serialize().unwrap();
        let decoded = Block::from_bytes(&bytes).unwrap();

        // hash check via Hashable trait
        assert_eq!(block.double_sha256(), decoded.double_sha256());

        // merkle root validation
        assert!(decoded.validate_merkle_root());

        // PoW validation (fake for testing)
        let pow_ok = Header::fake_validate_pow([0u8; 32], decoded.header.difficulty_compact);
        assert!(pow_ok);
    }

    #[test]
    fn test_block_display() {
        // create a transaction
        let tx = Transaction::new(vec![b"in".to_vec()], vec![b"out".to_vec()]);

        // create a header with a placeholder merkle root
        let header = Header::new(1, 123, 0x207fffff, 42, [0u8; HASH_SIZE], [0u8; 32]);

        // create the block; new_with_merkle computes the correct merkle root automatically
        let block = Block::new_with_merkle(header, vec![tx]);

        // use the Display impl
        let s = format!("{}", block);
        assert!(s.contains("Block("));
        assert!(s.contains("hash="));
    }

    #[test]
    fn test_merkle_root_consistency() {
        let tx1 = Transaction::new(vec![b"a".to_vec()], vec![b"b".to_vec()]);
        let tx2 = Transaction::new(vec![b"c".to_vec()], vec![b"d".to_vec()]);
        let txs = vec![tx1.clone(), tx2.clone()];

        let root1 = compute_merkle_root(&txs);
        let root2 = compute_merkle_root(&txs);

        assert_eq!(root1, root2);
    }
}