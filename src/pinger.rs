use std::cmp::Ordering;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::{task, time::Instant};
use tracing::{error, info};

use crate::{
    error::CosmopingError,
    parser::AddrBook,
    reporter::{Report, ReportLine},
};
use async_trait::async_trait;
use futures::future::join_all;
use tokio::time::{timeout, Duration};

#[async_trait]
pub trait Ping {
    async fn ping_addr_book_hosts(&self, addr_book: AddrBook) -> Result<Report, CosmopingError>;
}

pub struct AddrBookPinger {}

impl AddrBookPinger {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AddrBookPinger {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Ping for AddrBookPinger {
    async fn ping_addr_book_hosts(&self, addr_book: AddrBook) -> Result<Report, CosmopingError> {
        let handles: Vec<_> = addr_book
            .addrs
            .iter()
            .map(|address_info| {
                let ip = address_info.addr.ip.clone();
                let port = address_info.addr.port;
                let id = address_info.addr.id.clone();
                task::spawn(async move { Self::measure_latency(id, ip, port).await })
            })
            .collect();

        // Handle errors in the results
        let mut report_lines = join_all(handles)
            .await
            .into_iter()
            .filter(|r| r.is_ok())
            .filter_map(|r| r.unwrap())
            .collect::<Vec<_>>();

        report_lines.sort_by(|a, b| {
            match (&a.latency_in_milliseconds, &b.latency_in_milliseconds) {
                (Some(latency_a), Some(latency_b)) => latency_b.cmp(latency_a), // Descending order
                (Some(_), None) => Ordering::Less,                              // a comes before b
                (None, Some(_)) => Ordering::Greater,                           // b comes before a
                (None, None) => Ordering::Equal,                                // Both are None
            }
        });

        Ok(Report { report_lines })
    }
}

#[async_trait]
trait Latency: Sync + Send {
    async fn measure_latency(id: String, host: String, port: u16) -> Option<ReportLine>;
}

#[async_trait]
impl Latency for AddrBookPinger {
    async fn measure_latency(id: String, host: String, port: u16) -> Option<ReportLine> {
        info!("Measuring latency for  {} on {}:{}", &id, &host, &port);
        // Construct the socket address
        let socket_addr: SocketAddr = match format!("{}:{}", host, port).parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("Unable to connect to {} | {} ", host, e);
                return None;
            }
        };

        let start = Instant::now();
        let duration = Duration::from_secs(60); // 60 seconds timeout

        let dur = match timeout(duration, TcpStream::connect(socket_addr)).await {
            Ok(Ok(_)) => {
                let elapsed = start.elapsed();

                Some(elapsed.as_millis() as u64)
            }
            _ => None,
        };

        Some(ReportLine {
            id,
            host,
            port,
            latency_in_milliseconds: dur,
        })
    }
}
