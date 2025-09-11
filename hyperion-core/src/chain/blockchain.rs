use crate::block::{Block, Header, Serializable, Transaction};
use crate::block::block::compute_merkle_root;
use crate::crypto::{Hashable, HASH_SIZE};
use crate::error::blockchain::BlockchainError;
use crate::consensus::{adjust_difficulty, create_genesis_block};

use std::collections::VecDeque;
use bincode::{Encode, Decode};


#[derive(Encode, Decode)]
pub struct Blockchain {
    pub blocks: VecDeque<Block>,
}

impl Blockchain {
    /// Create a new blockchain with a genesis block
    pub fn new(genesis_block: Block) -> Self {
        let mut blocks = VecDeque::new();
        blocks.push_back(genesis_block);
        Self { blocks }
    }

    pub fn new_with_genesis() -> Self {
        let genesis = create_genesis_block();
        Self::new(genesis)
    }

    /// Get the latest block
    pub fn latest_block(&self) -> &Block {
        self.blocks.back().expect("Blockchain should have at least one block")
    }

    /// Add a new block to the chain
    pub fn add_block(&mut self, block: Block, skip_pow: bool) -> Result<(), BlockchainError> {
        let prev_hash = self.latest_block().double_sha256();
        if block.header.prev_hash != prev_hash {
            return Err(BlockchainError::InvalidPreviousHash);
        }

        block.validate_merkle_root().map_err(|_| BlockchainError::InvalidMerkleRoot)?;

        if !skip_pow {
            block.header.validate_pow().map_err(|_| BlockchainError::InvalidMerkleRoot)?;
        }

        self.blocks.push_back(block);
        Ok(())
    }

    /// Simple validation: check PoW and merkle roots for all blocks
    pub fn validate(&self) -> bool {
        self.validate_with_options(false)
    }

    /// Validate chain with option to skip PoW
    pub fn validate_with_options(&self, skip_pow: bool) -> bool {
        for (i, block) in self.blocks.iter().enumerate() {
            // Skip prev_hash check for genesis
            if i > 0 {
                let prev_block = &self.blocks[i - 1];
                if block.header.prev_hash != prev_block.double_sha256() {
                    return false;
                }
            }

            if block.validate_merkle_root().is_err() {
                return false;
            }

            if !skip_pow && block.header.validate_pow().is_err() {
                return false;
            }
        }

        true
    }

    pub fn create_block_template(
        &self,
        transactions: Vec<Transaction>,
        difficulty_compact: u32,
        timestamp: u32,
    ) -> Block {
        let prev_hash = self.latest_block().double_sha256();
        // compute merkle root for the transactions
        let merkle_root = compute_merkle_root(&transactions);

        let header = Header::new(
            1,                  // version
            timestamp,
            difficulty_compact,
            0,                    // nonce initially 0
            prev_hash,
            merkle_root,
        );

        Block::new(header, transactions)
    }

    pub fn mine_new_block(
        chain: &Blockchain,
        transactions: Vec<Transaction>,
        timestamp: u32,
    ) -> Block {
        // Compute current difficulty
        let difficulty = adjust_difficulty(chain);

        let mut block = chain.create_block_template(transactions, difficulty, timestamp);

        // Mine block (PoW)
        crate::consensus::mine_block(&mut block.header);

        block
    }

    // pub fn create_and_mine_block(
    //     &self,
    //     transactions: Vec<Transaction>,
    //     difficulty_compact: u32,
    //     timestamp: u32,
    // ) -> Block {
    //     let prev_hash = self.latest_block().double_sha256();
    //     let merkle_root = compute_merkle_root(&transactions);

    //     let header = Header::new(1, timestamp, difficulty_compact, 0, prev_hash, merkle_root);
    //     let mined_header = mine_block(header);
    //     Block::new(mined_header, transactions)
    // }

    /// Get block by height/index
    pub fn get_block_by_height(&self, height: usize) -> Option<&Block> {
        self.blocks.get(height)
    }

    /// Find a block by hash
    pub fn find_block(&self, hash: [u8; HASH_SIZE]) -> Option<&Block> {
        self.iter().find(|b| b.double_sha256() == hash)
    }

    /// Convenience: return number of block
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Convenience: check if empty
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item=&Block> {
        self.blocks.iter()
    }

    pub fn iter_rev(&self) -> impl DoubleEndedIterator<Item=&Block> {
        self.blocks.iter().rev()
    }
}

impl Serializable for Blockchain {}
