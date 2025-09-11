use crate::block::Serializable;
use crate::crypto::Hashable;
use crate::error::transaction::TransactionError;

use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};


pub type InputData = Vec<u8>;
pub type OutputData = Vec<u8>;

#[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<InputData>,
    pub outputs: Vec<OutputData>,
}

impl Transaction {
    pub fn new(inputs: Vec<InputData>, outputs: Vec<OutputData>) -> Result<Self, TransactionError> {
        if inputs.is_empty() {
            return Err(TransactionError::EmptyInputs);
        }

        if outputs.is_empty() {
            return Err(TransactionError::EmptyOutputs);
        }

        Ok(Self { inputs, outputs })
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
    //use super::*;
    use super::{Transaction, Hashable, Serializable};

    #[test]
    fn test_transaction_hash_deterministic() {
        let tx1 = Transaction::new(vec![b"in".to_vec()], vec![b"out".to_vec()]).expect("Failed to create tx1");
        let tx2 = Transaction::new(vec![b"in".to_vec()], vec![b"out".to_vec()]).expect("Failed to create tx2");
        assert_eq!(tx1.double_sha256(), tx2.double_sha256());
    }

    #[test]
    fn test_transaction_roundtrip() {
        let tx = Transaction::new(vec![b"a".to_vec()], vec![b"b".to_vec()]).expect("Failed to create tx");
        let bytes = tx.serialize().expect("Failed to serialize tx bytes");
        let decoded = Transaction::from_bytes(&bytes).expect("Failed to decode tx from bytes");
        assert_eq!(tx.double_sha256(), decoded.double_sha256());
    }
}
