use hyperion_core::{block::Transaction, crypto::Hashable};

pub struct Mempool {
    pub txs: Vec<Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self { txs: vec![] }
    }

    pub fn add_tx(&mut self, tx: Transaction) {
        self.txs.push(tx);
    }

    pub fn remove_tx(&mut self, tx_to_remove: &Transaction) {
        let target_hash = tx_to_remove.double_sha256();
        self.txs.retain(|existing_tx| {
            existing_tx.double_sha256() != target_hash
        });
    }

    pub fn is_empty(&self) -> bool {
        self.txs.is_empty()
    }

    pub fn get_next_transaction(&mut self, n: usize) -> Option<Vec<Transaction>> {
        if self.txs.is_empty() {
            return None;
        }

        let count = n.min(self.txs.len());
        let txs: Vec<_> = self.txs.drain(..count).collect();
        Some(txs)
    }

    pub fn len(&self) -> usize {
        self.txs.len()
    }

    /// Persist/load mempool
    pub fn save(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    pub fn load() -> Self {
        // load from disk or default
        Self::new()
    }
}