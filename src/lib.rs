use std::path::PathBuf;

use error::CosmopingError;
use parser::{AddrBookParser, Parse};
use pinger::{AddrBookPinger, Ping};
use reporter::{AddrBookReporter, Reporting};
use writer::{AddrBookWriter, Writing};

pub mod cli;
pub mod error;
pub mod parser;
pub mod pinger;
pub mod reporter;
pub mod writer;

pub async fn latency_report(
    path: PathBuf,
    output_path: Option<PathBuf>,
) -> Result<(), CosmopingError> {
    let addr_book = AddrBookParser::default().parse_addr_book(path)?;
    let latencies = AddrBookPinger::default()
        .ping_addr_book_hosts(addr_book)
        .await?;
    let _ = AddrBookReporter::default().report_addr_book(&latencies);
    AddrBookWriter::default().write_addr_book(output_path, &latencies)
}
