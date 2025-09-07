#[derive(Debug)]
pub enum BlockError {
    InvalidMerkleRoot,
    EmptyTransactions,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BlockError {}