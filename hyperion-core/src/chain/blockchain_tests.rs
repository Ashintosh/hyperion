#[cfg(test)]
mod tests {
    use crate::block::{block::compute_merkle_root, Block, Header, Transaction};
    use crate::crypto::{HASH_SIZE, Hashable};
    use crate::chain::blockchain::Blockchain;

    use std::collections::VecDeque;

    /// Helper: create a simple transaction
    fn make_tx() -> Transaction {
        Transaction::new(vec![b"in".to_vec()], vec![b"out".to_vec()]).expect("Failed to make new tx")
    }

    /// Helper: create a block with given previous hash
    fn make_block(prev_hash: [u8; HASH_SIZE], txs: Vec<Transaction>) -> Block {
        let merkle_root = compute_merkle_root(&txs);
        let header = Header::new(1, 123, 0x207fffff, 0, prev_hash, merkle_root);
        Block::new(header, txs)
    }

    /// Helper: create a default block with a single tx
    fn make_block_single(prev_hash: [u8; HASH_SIZE]) -> Block {
        let tx = make_tx();
        make_block(prev_hash, vec![tx])
    }

    #[test]
    fn test_genesis_block() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let chain = Blockchain::new(genesis.clone());

        assert_eq!(chain.len(), 1);
        assert_eq!(chain.latest_block().double_sha256(), genesis.double_sha256());
    }

    #[test]
    fn test_add_block() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let mut chain = Blockchain::new(genesis.clone());

        let block1 = make_block_single(genesis.double_sha256());
        chain.add_block(block1.clone(), true).expect("Failed to add block to chain");

        assert_eq!(chain.len(), 2);
        assert_eq!(chain.latest_block().double_sha256(), block1.double_sha256());
    }

    #[test]
    #[should_panic]
    fn test_invalid_block_rejection() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let mut chain = Blockchain::new(genesis.clone());

        let bad_block = make_block_single([1u8; HASH_SIZE]);
        chain.add_block(bad_block, true).expect("Rejected bad block"); // should panic
    }

    #[test]
    fn test_merkle_root_consistency() {
        let tx1 = make_tx();
        let tx2 = make_tx();
        let block = make_block([0u8; HASH_SIZE], vec![tx1.clone(), tx2.clone()]);
        let expected_root = compute_merkle_root(&vec![tx1, tx2]);

        assert_eq!(compute_merkle_root(&block.transactions), expected_root);
        assert!(block.validate_merkle_root().is_ok());
    }

    #[test]
    fn test_block_template_creation() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let chain = Blockchain::new(genesis.clone());

        let txs = vec![make_tx(), make_tx()];
        let block_template = chain.create_block_template(txs.clone(), 0x207fffff, 12345);

        // Check prev_hash
        assert_eq!(block_template.header.prev_hash, genesis.double_sha256());
        // Check merkle root
        assert_eq!(block_template.header.merkle_root, compute_merkle_root(&txs));
        // Nonce should start at 0
        assert_eq!(block_template.header.nonce, 0);
    }

    #[test]
    fn test_chain_lookup_and_iterators() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let mut chain = Blockchain::new(genesis.clone());

        let block1 = make_block_single(genesis.double_sha256());
        chain.add_block(block1.clone(), true).expect("Failed adding block1 to chain");

        let block2 = make_block_single(block1.double_sha256());
        chain.add_block(block2.clone(), true).expect("Failed adding block2 to chain");

        // get_block_by_height
        assert_eq!(chain.get_block_by_height(0).expect("Failed to get block1 by height").double_sha256(), genesis.double_sha256());
        assert_eq!(chain.get_block_by_height(2).expect("Failed to get block2 by height").double_sha256(), block2.double_sha256());
        assert!(chain.get_block_by_height(3).is_none());

        // find_block
        assert_eq!(chain.find_block(block1.double_sha256()).expect("Failed to find block1").double_sha256(), block1.double_sha256());
        assert!(chain.find_block([1u8; HASH_SIZE]).is_none());

        // iter and iter_rev
        let hashes: Vec<_> = chain.iter().map(|b| b.double_sha256()).collect();
        let rev_hashes: Vec<_> = chain.iter_rev().map(|b| b.double_sha256()).collect();
        assert_eq!(hashes[0], genesis.double_sha256());
        assert_eq!(rev_hashes[0], block2.double_sha256());
    }

    #[test]
    fn test_empty_transaction_block() {
        let block = make_block([0u8; HASH_SIZE], vec![]);
        // Merkle root should be zero
        assert_eq!(compute_merkle_root(&block.transactions), [0u8; HASH_SIZE]);
        assert!(block.validate_merkle_root().is_ok());
    }

    #[test]
    fn test_validate_with_skip_pow() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let mut chain = Blockchain::new(genesis.clone());

        let block1 = make_block_single(genesis.double_sha256());
        chain.add_block(block1.clone(), true).expect("Failed to add block to chain");

        // validate skipping PoW
        assert!(chain.validate_with_options(true));
    }

    #[test]
    fn test_invalid_merkle_root_detection() {
        let tx1 = make_tx();
        let tx2 = make_tx();
        let mut block = make_block([0u8; HASH_SIZE], vec![tx1.clone(), tx2.clone()]);
        block.header.merkle_root = [1u8; HASH_SIZE];

        let chain = Blockchain::new(block.clone());
        assert!(!chain.validate_with_options(true));
    }

    #[test]
    #[should_panic]
    fn test_invalid_prev_hash_detection() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let mut chain = Blockchain::new(genesis.clone());

        let block = make_block_single([1u8; HASH_SIZE]); // wrong prev_hash
        chain.add_block(block, true).expect("Prev hash invalid"); // should panic due to prev_hash mismatch
    }

    #[test]
    fn test_block_template_with_empty_transactions() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let chain = Blockchain::new(genesis.clone());

        let empty_txs = vec![];
        let block_template = chain.create_block_template(empty_txs.clone(), 0x1d00ffff, 9999);

        // prev_hash points to latest block
        assert_eq!(block_template.header.prev_hash, genesis.double_sha256());
        // merkle root should be zero for empty tx list
        assert_eq!(block_template.header.merkle_root, [0u8; HASH_SIZE]);
        // nonce starts at 0
        assert_eq!(block_template.header.nonce, 0);
    }

    #[test]
    fn test_find_block_returns_none_for_unknown_hash() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let chain = Blockchain::new(genesis.clone());

        let unknown_hash = [42u8; HASH_SIZE];
        assert!(chain.find_block(unknown_hash).is_none());
    }

    #[test]
    fn test_len_and_is_empty() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let chain = Blockchain::new(genesis.clone());

        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());

        // manually pop all blocks (simulate empty)
        let blocks_only_chain = Blockchain { blocks: VecDeque::new() };
        assert_eq!(blocks_only_chain.len(), 0);
        assert!(blocks_only_chain.is_empty());
    }

    #[test]
    fn test_iterators_on_empty_chain() {
        let empty_chain = Blockchain { blocks: VecDeque::new() };
        assert_eq!(empty_chain.iter().count(), 0);
        assert_eq!(empty_chain.iter_rev().count(), 0);
    }


    #[test]
    fn test_merkle_root_with_multiple_transactions() {
        let txs: Vec<Transaction> = (0..7)
            .map(|i| {
                let i_bytes = (i as u32).to_le_bytes().to_vec(); // u32 → [u8; 4] → Vec<u8>
                Transaction::new(vec![i_bytes.clone()], vec![i_bytes]).expect("Failed to create new tx") // wrap in Vec<Vec<u8>>
            })
            .collect();

        let block = make_block([0u8; HASH_SIZE], txs.clone());
        let expected_root = compute_merkle_root(&txs);
        assert_eq!(block.header.merkle_root, expected_root);
        assert!(block.validate_merkle_root().is_ok());
    }

    #[test]
    fn test_block_template_with_custom_difficulty() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let chain = Blockchain::new(genesis.clone());

        let txs = vec![make_tx()];
        let difficulty = 0x1d00ffff;
        let block_template = chain.create_block_template(txs.clone(), difficulty, 1000);

        assert_eq!(block_template.header.difficulty_compact, difficulty);
    }

    #[test]
    fn test_validate_fails_on_tampered_prev_hash() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let mut chain = Blockchain::new(genesis.clone());

        let mut block1 = make_block_single(genesis.double_sha256());
        chain.add_block(block1.clone(), true).expect("Failed to add block to chain");

        // Tamper with prev_hash after adding
        block1.header.prev_hash = [1u8; HASH_SIZE];
        chain.blocks.push_back(block1);

        assert!(!chain.validate_with_options(true));
    }

    #[test]
    fn test_iterators_order_consistency() {
        let genesis = make_block_single([0u8; HASH_SIZE]);
        let mut chain = Blockchain::new(genesis.clone());

        let block1 = make_block_single(genesis.double_sha256());
        chain.add_block(block1.clone(), true).expect("Failed to add block1 to chain");

        let block2 = make_block_single(block1.double_sha256());
        chain.add_block(block2.clone(), true).expect("Failed to add block2 to chain");

        let iter_hashes: Vec<_> = chain.iter().map(|b| b.double_sha256()).collect();
        let rev_iter_hashes: Vec<_> = chain.iter_rev().map(|b| b.double_sha256()).collect();

        assert_eq!(iter_hashes, vec![genesis.double_sha256(), block1.double_sha256(), block2.double_sha256()]);
        assert_eq!(rev_iter_hashes, vec![block2.double_sha256(), block1.double_sha256(), genesis.double_sha256()]);
    }
}
