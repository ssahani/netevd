// SPDX-License-Identifier: LGPL-3.0-or-later

//! systemd-networkd state file parsing

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

const SYSTEMD_NETIF_LINKS: &str = "/run/systemd/netif/links";
const SYSTEMD_NETIF_STATE: &str = "/run/systemd/netif/state";

/// Link state information from systemd-networkd
#[derive(Debug, Default, Clone)]
pub struct LinkState {
    pub admin_state: String,
    pub oper_state: String,
    pub carrier_state: String,
    pub address_state: String,
    pub ipv4_address_state: String,
    pub ipv6_address_state: String,
    pub online_state: String,
    pub dns: Vec<String>,
    pub domains: Vec<String>,
    pub gateway: Option<String>,
    pub gateway6: Option<String>,
}

/// Manager state information from systemd-networkd
#[derive(Debug, Default, Clone)]
pub struct ManagerState {
    pub operational_state: String,
    pub carrier_state: String,
    pub address_state: String,
    pub ipv4_address_state: String,
    pub ipv6_address_state: String,
    pub online_state: String,
}

/// Parse a flat KEY=VALUE file into a HashMap
fn parse_key_value_file(path: &std::path::Path) -> Result<HashMap<String, String>> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file {:?}: {}", path, e))?;

    let mut kv = HashMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            kv.insert(key.to_string(), value.to_string());
        }
    }
    Ok(kv)
}

/// Parse link state file from systemd-networkd
pub fn parse_link_state_file(ifindex: u32) -> Result<LinkState> {
    let path = PathBuf::from(SYSTEMD_NETIF_LINKS).join(ifindex.to_string());

    if !path.exists() {
        debug!("Link state file does not exist: {:?}", path);
        return Ok(LinkState::default());
    }

    let kv = parse_key_value_file(&path)?;

    let mut state = LinkState::default();

    state.admin_state = kv.get("ADMIN_STATE").cloned().unwrap_or_default();
    state.oper_state = kv.get("OPER_STATE").cloned().unwrap_or_default();
    state.carrier_state = kv.get("CARRIER_STATE").cloned().unwrap_or_default();
    state.address_state = kv.get("ADDRESS_STATE").cloned().unwrap_or_default();
    state.ipv4_address_state = kv.get("IPV4_ADDRESS_STATE").cloned().unwrap_or_default();
    state.ipv6_address_state = kv.get("IPV6_ADDRESS_STATE").cloned().unwrap_or_default();
    state.online_state = kv.get("ONLINE_STATE").cloned().unwrap_or_default();

    // Parse DNS servers (space-separated on a single line)
    if let Some(dns_str) = kv.get("DNS") {
        state.dns = dns_str.split_whitespace().map(|s| s.to_string()).collect();
    }

    // Parse domains (space-separated on a single line)
    if let Some(domains_str) = kv.get("DOMAINS") {
        state.domains = domains_str.split_whitespace().map(|s| s.to_string()).collect();
    }

    // Parse gateways
    state.gateway = kv.get("GATEWAY").cloned().filter(|s| !s.is_empty());
    state.gateway6 = kv.get("GATEWAY6").cloned().filter(|s| !s.is_empty());

    debug!("Parsed link state for ifindex {}: {:?}", ifindex, state);
    Ok(state)
}

/// Parse manager state file from systemd-networkd
pub fn parse_manager_state_file() -> Result<ManagerState> {
    let path = PathBuf::from(SYSTEMD_NETIF_STATE);

    if !path.exists() {
        debug!("Manager state file does not exist: {:?}", path);
        return Ok(ManagerState::default());
    }

    let kv = parse_key_value_file(&path)?;

    let mut state = ManagerState::default();

    state.operational_state = kv.get("OPER_STATE").cloned().unwrap_or_default();
    state.carrier_state = kv.get("CARRIER_STATE").cloned().unwrap_or_default();
    state.address_state = kv.get("ADDRESS_STATE").cloned().unwrap_or_default();
    state.ipv4_address_state = kv.get("IPV4_ADDRESS_STATE").cloned().unwrap_or_default();
    state.ipv6_address_state = kv.get("IPV6_ADDRESS_STATE").cloned().unwrap_or_default();
    state.online_state = kv.get("ONLINE_STATE").cloned().unwrap_or_default();

    debug!("Parsed manager state: {:?}", state);
    Ok(state)
}

