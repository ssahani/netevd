// SPDX-License-Identifier: LGPL-3.0-or-later

//! Network event watchers using netlink
//!
//! NOTE: This implementation uses periodic polling instead of real-time netlink multicast
//! subscriptions. This is a pragmatic solution that works but could be improved in the future
//! by implementing proper netlink multicast group subscriptions for lower latency and better
//! efficiency.

use anyhow::Result;
use rtnetlink::Handle;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, info, warn};

use super::{
    address::get_ipv4_addresses,
    route::{add_route, calculate_table_id, discover_gateway, remove_route},
    routing_rule::{add_routing_rule_from, add_routing_rule_to, remove_routing_rules},
    NetworkState,
};

// Polling interval for checking network changes
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Watch for address changes and update routing rules accordingly
pub async fn watch_addresses(
    handle: Handle,
    state: Arc<RwLock<NetworkState>>,
    routing_policy_interfaces: Vec<String>,
) -> Result<()> {
    info!("Starting address watcher (polling mode)");

    // Track addresses we've seen before
    let mut last_seen_addresses: HashSet<(u32, IpAddr)> = HashSet::new();

    let mut interval = time::interval(POLL_INTERVAL);

    loop {
        interval.tick().await;

        // Get current interfaces we're monitoring
        let interfaces_to_monitor: Vec<(String, u32)> = {
            let state_read = state.read().await;
            routing_policy_interfaces
                .iter()
                .filter_map(|name| {
                    state_read
                        .get_link_index(name)
                        .map(|idx| (name.clone(), idx))
                })
                .collect()
        };

        // Check addresses for each monitored interface
        for (link_name, ifindex) in interfaces_to_monitor {
            match get_ipv4_addresses(&handle, ifindex).await {
                Ok(addresses) => {
                    let current_addrs: HashSet<(u32, IpAddr)> = addresses
                        .iter()
                        .map(|addr| (ifindex, *addr))
                        .collect();

                    // Detect changes
                    if current_addrs != last_seen_addresses
                        .iter()
                        .filter(|(idx, _)| *idx == ifindex)
                        .copied()
                        .collect()
                    {
                        debug!(
                            "Address change detected on interface {} ({})",
                            link_name, ifindex
                        );

                        if addresses.is_empty() {
                            info!(
                                "No addresses on interface {}, cleaning up routing configuration",
                                link_name
                            );
                            if let Err(e) = drop_configuration(&handle, &state, ifindex).await {
                                warn!("Failed to drop configuration: {}", e);
                            }

                            // Remove old addresses from tracking
                            last_seen_addresses.retain(|(idx, _)| *idx != ifindex);
                        } else {
                            info!(
                                "Configuring routing rules for interface {} with {} addresses",
                                link_name,
                                addresses.len()
                            );
                            if let Err(e) =
                                configure_network(&handle, &state, ifindex, &addresses).await
                            {
                                warn!("Failed to configure network: {}", e);
                            }

                            // Update tracking
                            last_seen_addresses.retain(|(idx, _)| *idx != ifindex);
                            last_seen_addresses.extend(current_addrs);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get addresses for interface {}: {}", ifindex, e);
                }
            }
        }
    }
}

/// Watch for route changes
pub async fn watch_routes(_handle: Handle, _state: Arc<RwLock<NetworkState>>) -> Result<()> {
    info!("Starting route watcher (polling mode)");

    let mut interval = time::interval(POLL_INTERVAL);

    loop {
        interval.tick().await;

        // TODO: Implement route change detection
        // For now, this is a placeholder that could trigger script execution
        debug!("Route watcher tick");
    }
}

/// Watch for link changes and update network state
pub async fn watch_links(handle: Handle, state: Arc<RwLock<NetworkState>>) -> Result<()> {
    info!("Starting link watcher (polling mode)");

    let mut interval = time::interval(POLL_INTERVAL);

    loop {
        interval.tick().await;

        // Refresh link list
        match crate::network::link::acquire_links(&mut *state.write().await, &handle).await {
            Ok(_) => {
                debug!("Link list refreshed");
            }
            Err(e) => {
                warn!("Failed to refresh link list: {}", e);
            }
        }
    }
}

/// Configure routing rules and routes for an interface
async fn configure_network(
    handle: &Handle,
    state: &Arc<RwLock<NetworkState>>,
    ifindex: u32,
    addresses: &[IpAddr],
) -> Result<()> {
    let table = calculate_table_id(ifindex);

    // Discover gateway for this interface
    let gateway = match discover_gateway(handle, ifindex).await? {
        Some(gw) => gw,
        None => {
            warn!("No gateway found for interface {}", ifindex);
            return Ok(());
        }
    };

    // Add default route to custom table
    add_route(handle, ifindex, gateway, table).await?;

    // Add routing rules for each address
    for address in addresses {
        // Add "from" rule
        add_routing_rule_from(handle, *address, table).await?;

        // Add "to" rule
        add_routing_rule_to(handle, *address, table).await?;
    }

    // Update state in a single atomic write operation
    // This prevents race conditions where another watcher could modify state
    // between individual updates
    {
        let mut state_write = state.write().await;

        // Add all routing rules to state
        for address in addresses {
            state_write.add_routing_rule_from(*address, table);
            state_write.add_routing_rule_to(*address, table);
        }

        // Add route to state
        state_write.add_route(ifindex, table, Some(gateway));
    }

    info!(
        "Successfully configured routing for interface {} with {} addresses",
        ifindex,
        addresses.len()
    );

    Ok(())
}

/// Remove routing configuration for an interface
async fn drop_configuration(
    handle: &Handle,
    state: &Arc<RwLock<NetworkState>>,
    ifindex: u32,
) -> Result<()> {
    let table = calculate_table_id(ifindex);

    // Get addresses that need to be cleaned up
    let addresses_to_clean: Vec<IpAddr> = {
        let state_read = state.read().await;
        state_read
            .routing_rules_from
            .keys()
            .chain(state_read.routing_rules_to.keys())
            .filter(|addr| {
                // Find rules associated with this table
                state_read
                    .routing_rules_from
                    .get(addr)
                    .map(|rule| rule.table == table)
                    .unwrap_or(false)
                    || state_read
                        .routing_rules_to
                        .get(addr)
                        .map(|rule| rule.table == table)
                        .unwrap_or(false)
            })
            .copied()
            .collect()
    };

    // Remove routing rules for each address
    for address in &addresses_to_clean {
        remove_routing_rules(handle, *address, table).await?;

        // Update state
        let mut state_write = state.write().await;
        state_write.remove_routing_rules(address);
    }

    // Remove routes from table
    remove_route(handle, ifindex, table).await?;

    // Update state
    {
        let mut state_write = state.write().await;
        state_write.remove_route(ifindex, table);
    }

    info!("Dropped routing configuration for interface {}", ifindex);

    Ok(())
}
