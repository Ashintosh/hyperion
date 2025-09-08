use crate::chain::Blockchain;
use crate::block::{Block, Transaction};
use crate::consensus::{mine_block, adjust_difficulty};


/// High-level helper: create and mine a new block with given transactions
pub fn mine_new_block(
    chain: &Blockchain,
    txs: Vec<Transaction>,
    timestamp: u32,
) -> Block {
    let difficulty = adjust_difficulty(chain);
    let mut block = chain.create_block_template(txs, difficulty, timestamp);
    mine_block(&mut block.header);
    block
}