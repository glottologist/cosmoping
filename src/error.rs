use thiserror::Error;

// Define custom error types for Cosmoping-related operations
#[derive(Error, Debug)]
pub enum CosmopingError {
    #[error("Unable to parse address book file: {0}")]
    UnableToParseAddrBookFile(#[from] serde_json::Error),
    #[error("Address book path was not found: {0}")]
    AddrBookWasNotFound(#[from] std::io::Error),
    #[error("Address book path does not exist: {0}")]
    AddrBookPathDoesNotExist(String),
}
