use regex::Regex;
use reqwest::{Client, Response};
use serde::Deserialize;
use std::{error::Error, time::Duration};
use tracing::info;

use crate::{parser::AddressInfo, pinger::MonikerData};

#[derive(Debug, Deserialize)]
pub struct NetInfo {
    pub result: ResultData,
}

#[derive(Debug, Deserialize)]
pub struct ResultData {
    pub peers: Vec<Peer>,
}

#[derive(Debug, Deserialize)]
pub struct Peer {
    pub node_info: NodeInfo,
    pub remote_ip: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub network: String,
    pub moniker: String,
    pub listen_addr: String,
}

async fn parse_response(response: Response) -> Result<NetInfo, reqwest::Error> {
    let data: NetInfo = response.json().await?;
    Ok(data)
}

fn get_port_from_listen_addr(listen_addr: String) -> Option<u64> {
    let re = Regex::new(r":(\d+)$").unwrap();

    re.captures(&listen_addr)
        .map(|c| c.get(1))
        .flatten()
        .map(|p| p.as_str().parse::<u64>())
        .map(|p| p.unwrap_or(0u64))
}

fn convert_to_netinfo(ni: NetInfo, chain_id: &str) -> Vec<MonikerData> {
    let mut data: Vec<MonikerData> = Vec::new();
    for i in ni.result.peers {
        if i.node_info.network == chain_id {
            let port = get_port_from_listen_addr(i.node_info.listen_addr);
            let datum = MonikerData {
                id: i.node_info.id,
                moniker: i.node_info.moniker,
                remote_ip: i.remote_ip,
                port,
            };
            data.push(datum);
        }
    }
    data
}

pub async fn grab_net_info(client: &Client, host: String, port: u16) -> Option<NetInfo> {
    let url = format!("http://{}:{}/net_info", host, port);
    info!("Trying to get net_info on  {}", &url);
    match client.get(&url).send().await {
        Err(_) => None,
        Ok(r) => {
            if !r.status().is_success() {
                return None;
            }
            info!("Got net_info data on {}", url);
            return parse_response(r).await.ok();
        }
    }
}

pub async fn attempt_to_get_net_info(addr_info: &AddressInfo, chain_id: &str) -> Vec<MonikerData> {
    info!("Attempting to get net_info for  {}", &addr_info.addr.ip);
    let client = Client::builder()
        .timeout(Duration::new(5, 0)) // Set timeout to 5 seconds
        .build()
        .unwrap();
    match grab_net_info(&client, addr_info.addr.ip.clone(), 26657u16).await {
        Some(ni) => convert_to_netinfo(ni, chain_id),
        None => match grab_net_info(
            &client,
            addr_info.addr.ip.clone(),
            addr_info.addr.port + 1u16,
        )
        .await
        {
            Some(ni) => convert_to_netinfo(ni, chain_id),
            None => Vec::new(),
        },
    }
}
