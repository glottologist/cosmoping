use cosmoping::cli::{Cli, Command};
use tracing::info;
use clap::Parser;

/// Main asynchronous entry point
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Parse command-line arguments
    let cli = Cli::parse();
    dotenv::dotenv().ok();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
