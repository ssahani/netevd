// SPDX-License-Identifier: LGPL-3.0-or-later

//! systemd-networkd state file parsing

use anyhow::Result;
use configparser::ini::Ini;
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

/// Parse link state file from systemd-networkd
pub fn parse_link_state_file(ifindex: u32) -> Result<LinkState> {
    let path = PathBuf::from(SYSTEMD_NETIF_LINKS).join(ifindex.to_string());

    if !path.exists() {
        debug!("Link state file does not exist: {:?}", path);
        return Ok(LinkState::default());
    }

    let mut conf = Ini::new();
    conf.load(&path)
        .map_err(|e| anyhow::anyhow!("Failed to parse link state file {:?}: {}", path, e))?;

    let mut state = LinkState::default();

    // Parse various sections
    state.admin_state = conf.get("ADMIN_STATE", "AdminState").unwrap_or_default();

    state.oper_state = conf.get("OPER_STATE", "OperationalState").unwrap_or_default();
    state.carrier_state = conf.get("OPER_STATE", "CarrierState").unwrap_or_default();
    state.address_state = conf.get("OPER_STATE", "AddressState").unwrap_or_default();
    state.ipv4_address_state = conf.get("OPER_STATE", "IPv4AddressState").unwrap_or_default();
    state.ipv6_address_state = conf.get("OPER_STATE", "IPv6AddressState").unwrap_or_default();
    state.online_state = conf.get("OPER_STATE", "OnlineState").unwrap_or_default();

    // Parse DNS servers (configparser doesn't easily iterate, so check common indices)
    for i in 0..10 {
        if let Some(dns) = conf.get("DNS", &format!("DNS{}", i)) {
            state.dns.push(dns);
        }
    }

    // Parse domains
    for i in 0..10 {
        if let Some(domain) = conf.get("DOMAINS", &format!("Domain{}", i)) {
            state.domains.push(domain);
        }
    }

    // Parse routes/gateways
    state.gateway = conf.get("ROUTE", "Gateway");
    state.gateway6 = conf.get("ROUTE", "Gateway6");

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

    let mut conf = Ini::new();
    conf.load(&path)
        .map_err(|e| anyhow::anyhow!("Failed to parse manager state file {:?}: {}", path, e))?;

    let mut state = ManagerState::default();

    state.operational_state = conf.get("MANAGER_STATE", "OperationalState").unwrap_or_default();
    state.carrier_state = conf.get("MANAGER_STATE", "CarrierState").unwrap_or_default();
    state.address_state = conf.get("MANAGER_STATE", "AddressState").unwrap_or_default();
    state.ipv4_address_state = conf.get("MANAGER_STATE", "IPv4AddressState").unwrap_or_default();
    state.ipv6_address_state = conf.get("MANAGER_STATE", "IPv6AddressState").unwrap_or_default();
    state.online_state = conf.get("MANAGER_STATE", "OnlineState").unwrap_or_default();

    debug!("Parsed manager state: {:?}", state);
    Ok(state)
}

/// Get operational state for a link (key state indicator)
pub fn get_link_operational_state(ifindex: u32) -> String {
    parse_link_state_file(ifindex)
        .map(|state| state.oper_state)
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Check if link is in routable state
pub fn is_link_routable(ifindex: u32) -> bool {
    get_link_operational_state(ifindex) == "routable"
}
