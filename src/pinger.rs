use crate::{
    error::CosmopingError,
    parser::AddrBook,
    reporter::{Report, ReportLine},
};
use async_trait::async_trait;
use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;
use std::cmp::Ordering;
use tokio::task;
use tracing::{error, info};

#[async_trait]
pub trait Ping {
    async fn ping_addr_book_hosts(&self, addr_book: AddrBook) -> Result<Report, CosmopingError>;
}

pub struct AddrBookPinger {
    location_api_key: Option<String>,
}

impl AddrBookPinger {
    pub fn new(location_api_key: Option<String>) -> Self {
        Self { location_api_key }
    }
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GeoInfo {
    ip: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    loc: Option<String>,
    org: Option<String>,
    postal: Option<String>,
    timezone: Option<String>,
    readme: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct LatencyLine {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub latency_in_milliseconds: Option<u64>,
}

#[async_trait]
impl Ping for AddrBookPinger {
    async fn ping_addr_book_hosts(&self, addr_book: AddrBook) -> Result<Report, CosmopingError> {
        let handles: Vec<_> = addr_book
            .addrs
            .iter()
            .map(|ai| {
                let addr = ai.addr.clone();
                let lat_id = addr.id.clone();
                let lat_ip = addr.ip.clone();
                let lat_port = addr.port;
                let api_key = self.location_api_key.clone();
                task::spawn(async move {
                    Self::measure_latency(api_key.clone(), lat_id.clone(), lat_ip.clone(), lat_port)
                        .await
                })
            })
            .collect();

        // Handle errors in the results
        let mut report_lines = join_all(handles)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
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
    async fn measure_latency(
        api_key: Option<String>,
        id: String,
        host: String,
        port: u16,
    ) -> Option<ReportLine>;
}

#[async_trait]
impl Latency for AddrBookPinger {
    async fn measure_latency(
        api_key: Option<String>,
        id: String,
        host: String,
        port: u16,
    ) -> Option<ReportLine> {
        info!("Measuring latency for  {} on {}:{}", &id, &host, &port);
        let payload = [0; 8];
        let ip_host = host.parse().ok();

        let dur = if let Some(ip) = ip_host {
            match surge_ping::ping(ip, &payload).await {
                Ok((_p, d)) => Some(d.as_millis() as u64),
                _ => None,
            }
        } else {
            None
        };

        let geo = if dur.is_some() {
            if api_key.is_some() {
                info!("Getting location for  {} on {}:{}", &id, &host, &port);
                let token = api_key.unwrap();
                let url = format!("https://ipinfo.io/{}?token={}", host, token);

                let client = Client::new();
                match client.get(&url).send().await {
                    Ok(r) => {
                        if r.status().is_success() {
                            match r.json::<GeoInfo>().await {
                                Ok(g) => Some(g),
                                Err(e) => {
                                    error!(
                                        "Error getting location for {} | {} on {}",
                                        host, e, &url
                                    );
                                    None
                                }
                            }
                        } else {
                            error!(
                                "Error getting location for {} | {} on {}",
                                host,
                                r.status(),
                                &url
                            );
                            None
                        }
                    }
                    Err(e) => {
                        error!("Error getting location for {} | {}", host, e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };
        match geo {
            None => Some(ReportLine {
                id,
                host,
                port,
                latency_in_milliseconds: dur,
                city: None,
                country: None,
            }),
            Some(g) => Some(ReportLine {
                id,
                host,
                port,
                latency_in_milliseconds: dur,
                city: g.city,
                country: g.country,
            }),
        }
    }
}
