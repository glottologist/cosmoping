use std::path::PathBuf;

use crate::{error::CosmopingError, reporter::Report};

pub trait Writing {
    fn write_addr_book(
        &self,
        output_path: Option<PathBuf>,
        report: &Report,
    ) -> Result<(), CosmopingError>;
}
pub struct AddrBookWriter {}

impl AddrBookWriter {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for AddrBookWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writing for AddrBookWriter {
    fn write_addr_book(
        &self,
        output_path: Option<PathBuf>,
        report: &Report,
    ) -> Result<(), CosmopingError> {
        match output_path {
            None => Ok(()),
            Some(path) => {
                let _ = std::fs::write(&path, format!("{}", report));
                Ok(())
            }
        }
    }
}
