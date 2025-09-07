#[derive(Debug)]
pub enum BlockchainError {
    InvalidPreviousHash,
    InvalidMerkleRoot,
    InvalidPoW,
}

impl std::fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BlockchainError {}