use hyperion_core::block::Transaction;

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

    /// Persist/load mempool
    pub fn save(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    pub fn load() -> Self {
        // load from disk or default
        Self::new()
    }
}