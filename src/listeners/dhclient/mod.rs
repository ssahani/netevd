// SPDX-License-Identifier: LGPL-3.0-or-later

//! dhclient file watcher

pub mod parser;

use anyhow::{Context, Result};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use rtnetlink::Handle;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, info, warn};

use crate::bus::{hostnamed, resolved};
use crate::config::Config;
use crate::network::NetworkState;
use crate::system::execute;
use crate::system::paths::get_script_dir;
use parser::parse_lease_file;

const DHCLIENT_LEASE_FILE: &str = "/var/lib/dhclient/dhclient.leases";
const DEBOUNCE_DURATION: Duration = Duration::from_secs(2);

/// Watch dhclient lease file for changes
pub async fn watch_lease_file(
    config: Config,
    handle: Handle,
    state: Arc<RwLock<NetworkState>>,
) -> Result<()> {
    info!("Starting dhclient lease file watcher: {}", DHCLIENT_LEASE_FILE);

    // Check if lease file exists
    if !Path::new(DHCLIENT_LEASE_FILE).exists() {
        warn!("dhclient lease file not found: {}", DHCLIENT_LEASE_FILE);
        warn!("Waiting for lease file to be created...");
    }

    // Create channel for file system events
    let (tx, mut rx) = mpsc::channel(100);

    // Set up file watcher
    // IMPORTANT: Keep watcher alive for entire function lifetime
    let _watcher = {
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            NotifyConfig::default(),
        )
        .context("Failed to create file watcher")?;

        // Watch the parent directory (file might not exist yet)
        let watch_path = Path::new(DHCLIENT_LEASE_FILE)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid lease file path"))?;

        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .context("Failed to watch lease file directory")?;

        info!("Watching directory: {}", watch_path.display());
        watcher
    };

    // Process initial leases if file exists
    if Path::new(DHCLIENT_LEASE_FILE).exists() {
        if let Err(e) = process_lease_file(&config, &handle, &state).await {
            warn!("Failed to process initial lease file: {}", e);
        }
    }

    // Debounce timer to avoid processing rapid successive changes
    let mut debounce_timer = time::interval(DEBOUNCE_DURATION);
    debounce_timer.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    let mut pending_update = false;

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                // Check if the event is for our lease file
                let is_lease_file = event.paths.iter().any(|p| {
                    p.to_str().map(|s| s.contains("dhclient.leases")).unwrap_or(false)
                });

                if is_lease_file {
                    debug!("Lease file modified: {:?}", event.kind);
                    pending_update = true;
                }
            }
            _ = debounce_timer.tick() => {
                if pending_update {
                    pending_update = false;
                    if let Err(e) = process_lease_file(&config, &handle, &state).await {
                        warn!("Failed to process lease file: {}", e);
                    }
                }
            }
        }
    }
}

/// Process the lease file and execute scripts
async fn process_lease_file(
    config: &Config,
    _handle: &Handle,
    state: &Arc<RwLock<NetworkState>>,
) -> Result<()> {
    debug!("Processing lease file: {}", DHCLIENT_LEASE_FILE);

    // Parse lease file
    let leases = parse_lease_file(DHCLIENT_LEASE_FILE)
        .context("Failed to parse lease file")?;

    if leases.is_empty() {
        debug!("No active leases found");
        return Ok(());
    }

    // Process each lease
    for (interface, lease) in leases.iter() {
        info!("Processing DHCP lease for interface {}: {}", interface, lease.address);

        // Get interface index
        let ifindex_opt = {
            let state_read = state.read().await;
            state_read.get_link_index(interface)
        };

        let ifindex = match ifindex_opt {
            Some(idx) => idx,
            None => {
                debug!("Interface {} not found in state, skipping", interface);
                continue;
            }
        };

        // Send DNS to systemd-resolved if configured
        if config.network.use_dns && !lease.dns_servers.is_empty() {
            if let Err(e) = resolved::set_link_dns(ifindex, lease.dns_servers.clone()).await {
                warn!("Failed to set DNS for {}: {}", interface, e);
            } else {
                info!("Set DNS for {}: {:?}", interface, lease.dns_servers);
            }
        }

        // Send domain to systemd-resolved if configured
        if config.network.use_domain {
            if let Some(domain) = &lease.domain_name {
                let domains = vec![domain.clone()];
                if let Err(e) = resolved::set_link_domains(ifindex, domains).await {
                    warn!("Failed to set domain for {}: {}", interface, e);
                } else {
                    info!("Set domain for {}: {}", interface, domain);
                }
            }
        }

        // Send hostname to systemd-hostnamed if configured
        if config.network.use_hostname {
            if let Some(hostname) = &lease.hostname {
                if let Err(e) = hostnamed::set_static_hostname(hostname).await {
                    warn!("Failed to set hostname: {}", e);
                } else {
                    info!("Set hostname: {}", hostname);
                }
            }
        }

        // Execute scripts in routable.d/
        let script_dir = get_script_dir("routable");
        let mut env_vars = HashMap::new();
        env_vars.insert("LINK".to_string(), interface.clone());
        env_vars.insert("LINKINDEX".to_string(), ifindex.to_string());
        env_vars.insert("STATE".to_string(), "routable".to_string());
        env_vars.insert("BACKEND".to_string(), "dhclient".to_string());
        env_vars.insert("ADDRESSES".to_string(), lease.address.clone());

        // Add DHCP-specific variables
        env_vars.insert("DHCP_ADDRESS".to_string(), lease.address.clone());

        if let Some(mask) = &lease.subnet_mask {
            env_vars.insert("DHCP_SUBNET_MASK".to_string(), mask.clone());
        }

        if !lease.routers.is_empty() {
            env_vars.insert("DHCP_GATEWAY".to_string(), lease.routers.join(" "));
        }

        if !lease.dns_servers.is_empty() {
            env_vars.insert("DHCP_DNS".to_string(), lease.dns_servers.join(" "));
        }

        if let Some(domain) = &lease.domain_name {
            env_vars.insert("DHCP_DOMAIN".to_string(), domain.clone());
        }

        if let Some(hostname) = &lease.hostname {
            env_vars.insert("DHCP_HOSTNAME".to_string(), hostname.clone());
        }

        // Execute scripts
        if let Err(e) = execute::execute_scripts(&script_dir, env_vars).await {
            warn!("Failed to execute scripts in {}: {}", script_dir, e);
        }

        // Handle routing policy rules if configured
        let routing_policy_interfaces = config.network.get_routing_policy_interfaces();
        if routing_policy_interfaces.contains(interface) {
            info!(
                "Interface {} is in routing policy list, routing configuration will be handled by address watcher",
                interface
            );
        }
    }

    Ok(())
}
