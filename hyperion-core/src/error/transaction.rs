#[derive(Debug)]
pub enum TransactionError {
    EmptyInputs,
    EmptyOutputs,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TransactionError {}