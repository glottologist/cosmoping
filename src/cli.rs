use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::error::CosmopingError;

fn rationalize_path(input: &str) -> Result<PathBuf, CosmopingError> {
    let path = PathBuf::from(input);
    let resolved_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .map_err(CosmopingError::AddrBookPathWasNotFound)?
            .join(path)
    };

    if !resolved_path.exists() {
        return Err(CosmopingError::AddrBookPathDoesNotExist(input.to_string()));
    }
    Ok(resolved_path)
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
    #[arg(short, long, value_parser = rationalize_path)]
    pub addrbook_path: PathBuf,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(aliases = ["l"])]
    Latency(AddrBookArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_latency_command_with_alias() {
        let args = Cli::parse_from(&["cosmoping", "l", "--addrbook-path", "./addrbook.json"]);

        match args.command {
            Command::Latency(addr_book_args) => {
                assert_eq!(addr_book_args.addrbook_path, "./addrbook.json");
            }
        }
    }
}
