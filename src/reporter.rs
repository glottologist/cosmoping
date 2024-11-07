use std::fmt::Display;

use serde::Deserialize;
use tracing::info;

use crate::error::CosmopingError;

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Report {
    pub report_lines: Vec<ReportLine>,
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Latency Report")?;
        writeln!(f)?;

        // Write the table header
        writeln!(f, "| IP Address | Port | ID | Latency |")?;
        writeln!(f, "|------------|------|----|---------|")?;

        for report_line in &self.report_lines {
            writeln!(f, "{}", report_line)?;
        }
        Ok(())
    }
}

impl Display for ReportLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.latency_in_milliseconds {
            Some(latency) => write!(
                f,
                "| {} | {} | {} | {} |",
                self.host, self.port, self.id, latency
            ),
            None => write!(
                f,
                "| {} | {} | {} | Unreachable |",
                self.host, self.port, self.id
            ),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ReportLine {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub latency_in_milliseconds: Option<u64>,
}

pub trait Reporting {
    fn report_addr_book(&self, report: &Report) -> Result<(), CosmopingError>;
}

pub struct AddrBookReporter {}

impl AddrBookReporter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AddrBookReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporting for AddrBookReporter {
    fn report_addr_book(&self, report: &Report) -> Result<(), CosmopingError> {
        info!("{}", report);

        Ok(())
    }
}
