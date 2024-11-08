use std::{fs::File, io::BufReader, path::PathBuf};

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::CosmopingError;

#[derive(Serialize, Deserialize, Debug)]
pub struct AddrBook {
    pub key: String,
    pub addrs: Vec<AddressInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddressInfo {
    pub addr: NodeInfo,
    pub src: NodeInfo,
    pub buckets: Vec<u32>,
    pub attempts: u32,
    pub bucket_type: u32,
    pub last_attempt: String,
    pub last_success: String,
    pub last_ban_time: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub ip: String,
    pub port: u16,
}

pub trait Parse {
    fn parse_addr_book(&self, addr_book_path: PathBuf) -> Result<AddrBook, CosmopingError>;
}

pub struct AddrBookParser {}

impl AddrBookParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AddrBookParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parse for AddrBookParser {
    fn parse_addr_book(&self, addr_book_path: PathBuf) -> Result<AddrBook, CosmopingError> {
        let file = File::open(addr_book_path).map_err(CosmopingError::AddrBookWasNotFound)?;
        let reader = BufReader::new(file);
        let addr_book: AddrBook =
            serde_json::from_reader(reader).map_err(CosmopingError::UnableToParseAddrBookFile)?;
        info!("Address books has {} hosts", &addr_book.addrs.len());
        Ok(addr_book)
    }
}
