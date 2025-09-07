use hyperion_core::chain::blockchain::Blockchain;
use hyperion_core::block::Transaction;
use hyperion_core::crypto::Hashable;

fn main() {
    println!("Starting blockchain...");

    let mut chain = Blockchain::new_with_genesis();

    let tx = Transaction::new(vec![b"in".to_vec()], vec![b"out".to_vec()])
        .expect("tx build failed");

    let block = chain.create_and_mine_block(vec![tx], 0x207fffff, 12345);
    chain.add_block(block, false).expect("block add failed");

    println!("Blockchain length: {}", chain.len());
    println!("Latest block hash: {:?}", chain.latest_block().double_sha256());
}
