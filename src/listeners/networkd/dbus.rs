// SPDX-License-Identifier: LGPL-3.0-or-later

//! DBus signal handling for systemd-networkd

use anyhow::{Context, Result};
use futures::stream::StreamExt;
use rtnetlink::Handle;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use zbus::Connection;

use crate::audit::{AuditLogger, AuditResult};
use crate::bus::{hostnamed, resolved};
use crate::config::Config;
use crate::filters::{EventFilter, NetworkEvent};
use crate::metrics::MetricsHandle;
use crate::network::{address::get_all_addresses, NetworkState};
use crate::system::execute;
use crate::system::paths::get_script_dir;

use super::api::parse_link_state_file;
use super::json::build_link_describe_json;

const NETWORKD_LINK_PREFIX: &str = "/org/freedesktop/network1/link/";

/// Decode a systemd-networkd DBus link path to extract the ifindex.
/// systemd-networkd encodes the ifindex as a decimal string where each
/// character is represented as _XX (hex of ASCII code).
/// e.g. ifindex 2 -> "_32", ifindex 12 -> "_31_32"
fn decode_networkd_ifindex(path: &str) -> Option<u32> {
    let encoded = path.strip_prefix(NETWORKD_LINK_PREFIX)?;
    let mut decoded = String::new();
    let mut chars = encoded.chars();
    while let Some(c) = chars.next() {
        if c == '_' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() != 2 {
                return None;
            }
            let byte = u8::from_str_radix(&hex, 16).ok()?;
            decoded.push(byte as char);
        } else {
            decoded.push(c);
        }
    }
    decoded.parse().ok()
}

/// Start systemd-networkd DBus listener
pub async fn listen_networkd(
    config: Config,
    handle: Handle,
    state: Arc<RwLock<NetworkState>>,
    metrics: Option<MetricsHandle>,
    audit: Arc<AuditLogger>,
) -> Result<()> {
    info!("Starting systemd-networkd DBus listener");

    let connection = Connection::system()
        .await
        .context("Failed to connect to system bus")?;

    // Subscribe to PropertiesChanged signals from networkd
    let mut stream = zbus::MessageStream::from(&connection);

    // Track last seen state for each interface to avoid duplicate processing
    let mut last_states: HashMap<u32, String> = HashMap::new();

    while let Some(msg) = stream.next().await {
        if let Ok(msg) = msg {
            // Check if this is a PropertiesChanged signal from networkd
            let signal = msg.header();
            if signal.member().map(|m| m.as_str()) == Some("PropertiesChanged") {
                if let Some(path) = signal.path().map(|p| p.as_str()) {
                    // Check if this is a link signal from networkd
                    if path.starts_with(NETWORKD_LINK_PREFIX) {
                        if let Some(ifindex) = decode_networkd_ifindex(path) {
                            if let Err(e) = handle_link_signal(
                                &config,
                                &handle,
                                &state,
                                ifindex,
                                &mut last_states,
                                &metrics,
                                &audit,
                            )
                            .await
                            {
                                warn!("Error handling link signal for ifindex {}: {}", ifindex, e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle link PropertiesChanged signal
async fn handle_link_signal(
    config: &Config,
    handle: &Handle,
    state: &Arc<RwLock<NetworkState>>,
    ifindex: u32,
    last_states: &mut HashMap<u32, String>,
    metrics: &Option<MetricsHandle>,
    audit: &Arc<AuditLogger>,
) -> Result<()> {
    // Get link name
    let link_name = {
        let state_read = state.read().await;
        state_read
            .get_link_name(ifindex)
            .cloned()
            .unwrap_or_else(|| format!("unknown{}", ifindex))
    };

    // Parse link state from systemd-networkd
    let link_state = parse_link_state_file(ifindex)?;

    // Check if operational state changed
    let current_state = link_state.oper_state.clone();
    if let Some(last_state) = last_states.get(&ifindex) {
        if last_state == &current_state {
            debug!(
                "State unchanged for interface {} ({}): {}",
                link_name, ifindex, current_state
            );
            return Ok(());
        }
    }

    last_states.insert(ifindex, current_state.clone());

    info!(
        "Link {} ({}) state changed to: {}",
        link_name, ifindex, current_state
    );

    // Record metrics for state change
    if let Some(ref m) = metrics {
        m.interface_state_changes
            .with_label_values(&[link_name.as_str(), current_state.as_str()])
            .inc();
        m.events_total
            .with_label_values(&[current_state.as_str(), link_name.as_str(), "systemd-networkd"])
            .inc();
    }

    // Log audit event
    audit.log_network_event(
        &link_name,
        &current_state,
        AuditResult::Success,
        None,
    );

    // Get addresses for this interface
    let addresses = get_all_addresses(handle, ifindex)
        .await
        .unwrap_or_default();
    let address_strings: Vec<String> = addresses.iter().map(|a| a.to_string()).collect();

    // Build JSON once if enabled, reuse for logging and env var
    let json_value = if config.get_emit_json() {
        match build_link_describe_json(ifindex, link_name.clone(), &link_state, address_strings.clone()) {
            Ok(json) => {
                debug!("Link describe JSON: {}", json);
                Some(json)
            }
            Err(e) => {
                warn!("Failed to build link describe JSON: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Handle systemd-resolved integration
    if config.get_use_dns() && !link_state.dns.is_empty() {
        if let Err(e) = resolved::set_link_dns(ifindex, link_state.dns.clone()).await {
            warn!("Failed to set DNS for interface {}: {}", ifindex, e);
        }
    }

    if config.get_use_domain() && !link_state.domains.is_empty() {
        if let Err(e) = resolved::set_link_domains(ifindex, link_state.domains.clone()).await {
            warn!("Failed to set domains for interface {}: {}", ifindex, e);
        }
    }

    // Handle hostname
    if config.get_use_hostname() {
        // Try to extract hostname from domains
        if let Some(hostname) = link_state.domains.first() {
            if crate::system::validation::validate_hostname(hostname) {
                if let Err(e) = hostnamed::set_static_hostname(hostname).await {
                    warn!("Failed to set hostname: {}", e);
                }
            } else {
                warn!("Rejected invalid hostname from DHCP: {}", hostname);
            }
        }
    }

    // Execute scripts for this state (with filtering)
    let script_dir = get_script_dir(&current_state);
    if !current_state.is_empty() {
        // Create event filter from config
        let event_filter = EventFilter {
            filters: config.filters.clone(),
        };

        // Create network event for filtering
        let network_event = NetworkEvent {
            interface: link_name.clone(),
            event_type: current_state.clone(),
            backend: "systemd-networkd".to_string(),
            addresses: addresses.clone(),
            has_gateway: link_state.oper_state == "routable",
            dns_servers: link_state.dns.iter()
                .filter_map(|s| s.parse().ok())
                .collect(),
        };

        // Check if scripts should be executed based on filters
        if event_filter.should_execute(&network_event) {
            debug!("Event passed filters, executing scripts for {}", link_name);

            let mut env_vars = HashMap::new();
            env_vars.insert("LINK".to_string(), link_name.clone());
            env_vars.insert("LINKINDEX".to_string(), ifindex.to_string());
            env_vars.insert("STATE".to_string(), current_state.clone());
            env_vars.insert("BACKEND".to_string(), "systemd-networkd".to_string());
            env_vars.insert("ADDRESSES".to_string(), address_strings.join(" "));

            // Add JSON if enabled (reuse pre-built value)
            if let Some(ref json) = json_value {
                if let Ok(json_str) = serde_json::to_string(json) {
                    env_vars.insert("JSON".to_string(), json_str);
                }
            }

            if let Err(e) = execute::execute_scripts(&script_dir, env_vars).await {
                warn!("Failed to execute scripts in {}: {}", &script_dir, e);
            }
        } else {
            debug!("Event filtered out, skipping script execution for {}", link_name);
        }
    }

    // Handle routing policy rules for routable state
    if link_state.oper_state == "routable" {
        let routing_policy_interfaces = config.routing.get_routing_policy_interfaces();
        if routing_policy_interfaces.contains(&link_name) {
            info!(
                "Interface {} is routable and in routing policy list, configuring routing",
                link_name
            );

            // Configure routing rules (this will be handled by the address watcher)
            // The address watcher will detect the addresses and configure routing
            debug!("Routing configuration will be handled by address watcher");
        }
    }

    Ok(())
}
