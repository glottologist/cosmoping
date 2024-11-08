use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::error::CosmopingError;

fn rationalize_path(input: &str, check_exists: bool) -> Result<PathBuf, CosmopingError> {
    let path = PathBuf::from(input);
    let resolved_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .map_err(CosmopingError::AddrBookWasNotFound)?
            .join(path)
    };

    if !resolved_path.exists() && check_exists {
        return Err(CosmopingError::AddrBookPathDoesNotExist(input.to_string()));
    }
    Ok(resolved_path)
}
fn rationalize_addr_path(input: &str) -> Result<PathBuf, CosmopingError> {
    rationalize_path(input, true)
}
fn rationalize_optional_path(input: &str) -> Result<PathBuf, CosmopingError> {
    rationalize_path(input, false)
}

#[derive(Parser)]
#[clap(
    author,
    version,
    about = "Cosmoping",
    long_about = "Cosmoping latency report"
)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args)]
pub struct AddrBookArgs {
    #[arg(short, long, value_parser = rationalize_addr_path)]
    pub addrbook_path: PathBuf,

    #[arg(short, long, value_parser = rationalize_optional_path)]
    pub output_path: Option<PathBuf>,

    #[arg(short, long)]
    pub location_api_key: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(aliases = ["l"])]
    Latency(AddrBookArgs),
}
