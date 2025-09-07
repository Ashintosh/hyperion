#[derive(Debug)]
pub enum NetError {
    SerializationError,
    DeserializationError,
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for NetError {}