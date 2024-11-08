use crate::{
    error::CosmopingError,
    net_info::attempt_to_get_net_info,
    parser::AddrBook,
    reporter::{Report, ReportLine},
};
use async_trait::async_trait;
use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;
use std::{cmp::Ordering, collections::HashMap};
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MonikerData {
    pub id: String,
    pub moniker: String,
    pub remote_ip: String,
    pub port: Option<u64>,
}

#[async_trait]
impl Ping for AddrBookPinger {
    async fn ping_addr_book_hosts(&self, addr_book: AddrBook) -> Result<Report, CosmopingError> {
        let mut monikers: HashMap<String, MonikerData> = HashMap::new();

        for addr in addr_book.addrs.iter() {
            let net_info = attempt_to_get_net_info(addr).await;
            for md in net_info {
                if !monikers.contains_key(&md.id) {
                    monikers.insert(md.id.clone(), md);
                }
            }
        }

        for addr in addr_book.addrs.iter() {
            if !monikers.contains_key(&addr.addr.id) {
                monikers.insert(
                    addr.addr.id.clone(),
                    MonikerData {
                        id: addr.addr.id.clone(),
                        moniker: "Unknown".to_string(),
                        remote_ip: addr.addr.ip.clone(),
                        port: Some(addr.addr.port as u64),
                    },
                );
            }
        }

        let mut rep_opts: Vec<Option<ReportLine>> = Vec::new();
        for addr in monikers.values() {
            let lat_id = addr.id.clone();
            let lat_ip = addr.remote_ip.clone();
            let api_key = self.location_api_key.clone();
            let rep = Self::measure_latency(
                api_key.clone(),
                lat_id.clone(),
                lat_ip.clone(),
                addr.moniker.clone(),
                addr.port,
            )
            .await;
            rep_opts.push(rep);
        }

        // Handle errors in the results
        let mut report_lines = rep_opts.iter().flatten().cloned().collect::<Vec<_>>();

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
        moniker: String,
        port: Option<u64>,
    ) -> Option<ReportLine>;
}

#[async_trait]
impl Latency for AddrBookPinger {
    async fn measure_latency(
        api_key: Option<String>,
        id: String,
        host: String,
        moniker: String,
        port: Option<u64>,
    ) -> Option<ReportLine> {
        info!("Measuring latency for  {}:{} on {}", &id, &moniker, &host);
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
                info!("Getting location for  {} on {}", &id, &host);
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
                moniker,
                port,
                latency_in_milliseconds: dur,
                city: None,
                country: None,
            }),
            Some(g) => Some(ReportLine {
                id,
                host,
                moniker,
                port,
                latency_in_milliseconds: dur,
                city: g.city,
                country: g.country,
            }),
        }
    }
}
