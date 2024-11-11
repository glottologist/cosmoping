use clap::Parser;
use cosmoping::cli::{Cli, Command};
use tracing::info;

/// Main asynchronous entry point
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    use Command::*;
    match cli.command {
        Latency(ab) => {
            info!("Running latency report for {}", ab.addrbook_path.display());
            cosmoping::latency_report(
                ab.addrbook_path,
                ab.chain_id,
                ab.output_path,
                ab.location_api_key,
            )
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
