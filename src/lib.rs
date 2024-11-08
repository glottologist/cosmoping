use error::CosmopingError;
use parser::{AddrBookParser, Parse};
use pinger::{AddrBookPinger, Ping};
use reporter::{AddrBookReporter, Reporting};
use std::path::PathBuf;
use tracing::info;
use writer::{AddrBookWriter, Writing};
pub mod cli;
pub mod error;
pub mod net_info;
pub mod parser;
pub mod pinger;
pub mod reporter;
pub mod writer;

pub async fn latency_report(
    path: PathBuf,
    output_path: Option<PathBuf>,
    api_key_opt: Option<String>,
) -> Result<(), CosmopingError> {
    if api_key_opt.is_none() {
        info!("No location api key defined in the environment.  No locations will be sourced");
    }
    let addr_book = AddrBookParser::default().parse_addr_book(path)?;
    let latencies = AddrBookPinger::new(api_key_opt)
        .ping_addr_book_hosts(addr_book)
        .await?;
    let _ = AddrBookReporter::default().report_addr_book(&latencies);
    AddrBookWriter::default().write_addr_book(output_path, &latencies)
}
