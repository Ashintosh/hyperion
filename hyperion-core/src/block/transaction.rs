use crate::block::Serializable;
use crate::crypto::Hashable;

use bincode::{Encode, Decode};


pub type InputData = Vec<u8>;
pub type OutputData = Vec<u8>;

#[derive(Encode, Decode, Clone)]
pub struct Transaction {
    inputs: Vec<InputData>,
    outputs: Vec<OutputData>,
}

impl Transaction {
    pub fn new(inputs: Vec<InputData>, outputs: Vec<OutputData>) -> Self {
        Self { inputs, outputs }
    }
}

impl Serializable for Transaction {}
impl Hashable for Transaction {}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tx(inputs={}, output={}, hash={:x?})",
            self.inputs.len(),
            self.outputs.len(),
            hex::encode(self.double_sha256())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash_deterministic() {
        let tx1 = Transaction::new(vec![b"in".to_vec()], vec![b"out".to_vec()]);
        let tx2 = Transaction::new(vec![b"in".to_vec()], vec![b"out".to_vec()]);
        assert_eq!(tx1.double_sha256(), tx2.double_sha256());
    }

    #[test]
    fn test_transaction_roundtrip() {
        let tx = Transaction::new(vec![b"a".to_vec()], vec![b"b".to_vec()]);
        let bytes = tx.serialize().unwrap();
        let decoded = Transaction::from_bytes(&bytes).unwrap();
        assert_eq!(tx.double_sha256(), decoded.double_sha256());
    }
}
