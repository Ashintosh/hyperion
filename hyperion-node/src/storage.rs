use std::fs;
use hyperion_core::chain::blockchain::Blockchain;
use hyperion_core::block::Serializable;

pub fn save_chain(chain: &Blockchain) -> std::io::Result<()> {
    let bytes = chain.serialize().unwrap();
    fs::write("blockchain.dat", bytes)
}

pub fn load_chain() -> std::io::Result<Blockchain> {
    let bytes = fs::read("blockchain.dat")?;
    Ok(Blockchain::from_bytes(&bytes).unwrap())
}