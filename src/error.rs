use std::io;
use thiserror::Error;

// Define custom error types for Cosmoping-related operations
#[derive(Error, Debug)]
pub enum CosmopingError {
    #[error("Address book path was not found: {0}")]
    AddrBookPathWasNotFound(#[from] std::io::Error),
    #[error("Address book path does not exist: {0}")]
    AddrBookPathDoesNotExist(String),
}
