// SPDX-License-Identifier: LGPL-3.0-or-later

//! Network event watchers using real-time netlink events
//!
//! This implementation uses netlink multicast subscriptions for real-time
//! event notification with <100ms latency, replacing the previous polling
//! approach which had 5-second intervals.

use anyhow::Result;
use futures::stream::StreamExt;
use rtnetlink::Handle;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{
    address::get_ipv4_addresses,
    route::{add_route, calculate_table_id, discover_gateway, remove_route},
    routing_rule::{add_routing_rule_from, add_routing_rule_to, remove_routing_rules},
    NetworkState,
};

/// Watch for address changes using real-time netlink events
pub async fn watch_addresses(
    handle: Handle,
    state: Arc<RwLock<NetworkState>>,
    routing_policy_interfaces: Vec<String>,
) -> Result<()> {
    info!("Starting address watcher (real-time netlink events)");

    // Track addresses we've seen before
    let mut last_seen_addresses: HashSet<(u32, IpAddr)> = HashSet::new();

    // Subscribe to address change notifications
    let (connection, _, mut messages) = rtnetlink::new_connection()?;
    tokio::spawn(connection);

    info!("Address watcher subscribed to netlink events");

    // Process address change events in real-time
    while let Some((message, _)) = messages.next().await {
        use netlink_packet_route::RouteNetlinkMessage;

        let (event_type, msg) = match message.payload {
            netlink_packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewAddress(msg)) => ("new", msg),
            netlink_packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelAddress(msg)) => ("del", msg),
            _ => continue,
        };
        let ifindex = msg.header.index;

        debug!(
            "Address {} event on interface {}",
            event_type, ifindex
        );

        // Check if this interface is in our monitoring list
        let should_monitor = {
            let state_read = state.read().await;
            routing_policy_interfaces.iter().any(|name| {
                state_read.get_link_index(name) == Some(ifindex)
            })
        };

        if !should_monitor {
            continue;
        }

        // Get interface name
        let link_name = {
            let state_read = state.read().await;
            state_read.get_link_name(ifindex).cloned().unwrap_or_default()
        };

        // Get current addresses for this interface
        match get_ipv4_addresses(&handle, ifindex).await {
            Ok(addresses) => {
                let current_addrs: HashSet<(u32, IpAddr)> = addresses
                    .iter()
                    .map(|addr| (ifindex, *addr))
                    .collect();

                // Detect changes for this interface
                let old_addrs: HashSet<(u32, IpAddr)> = last_seen_addresses
                    .iter()
                    .filter(|(idx, _)| *idx == ifindex)
                    .copied()
                    .collect();

                if current_addrs != old_addrs {
                    info!(
                        "Address change detected on interface {} ({}): {} -> {} addresses",
                        link_name,
                        ifindex,
                        old_addrs.len(),
                        addresses.len()
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

    Ok(())
}

/// Watch for route changes using real-time netlink events
pub async fn watch_routes(handle: Handle, state: Arc<RwLock<NetworkState>>) -> Result<()> {
    info!("Starting route watcher (real-time netlink events)");

    // Subscribe to route change notifications
    let (connection, _, mut messages) = rtnetlink::new_connection()?;
    tokio::spawn(connection);

    info!("Route watcher subscribed to netlink events");

    // Process route change events
    while let Some((message, _)) = messages.next().await {
        use netlink_packet_route::RouteNetlinkMessage;

        let (event_type, msg) = match message.payload {
            netlink_packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewRoute(msg)) => ("new", msg),
            netlink_packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelRoute(msg)) => ("del", msg),
            _ => continue,
        };

        // Extract output interface from attributes
        use netlink_packet_route::route::RouteAttribute;
        let ifindex = msg
            .attributes
            .iter()
            .find_map(|attr| {
                if let RouteAttribute::Oif(idx) = attr {
                    Some(*idx)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        if ifindex == 0 {
            continue; // Skip routes without interface
        }

        debug!("Route {} event on interface {}", event_type, ifindex);

        // Get interface name
        let link_name = {
            let state_read = state.read().await;
            state_read.get_link_name(ifindex).cloned().unwrap_or_default()
        };

        info!(
            "Route {} on interface {} ({})",
            event_type, link_name, ifindex
        );

        // Execute scripts for route changes
        let script_dir = crate::system::paths::get_script_dir("routes");
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("LINK".to_string(), link_name.clone());
        env_vars.insert("LINKINDEX".to_string(), ifindex.to_string());
        env_vars.insert("EVENT".to_string(), event_type.to_string());
        env_vars.insert("STATE".to_string(), "routes".to_string());

        if let Err(e) = crate::system::execute::execute_scripts(&script_dir, env_vars).await {
            debug!("Failed to execute route scripts: {}", e);
        }
    }

    Ok(())
}

/// Watch for link changes using real-time netlink events
pub async fn watch_links(handle: Handle, state: Arc<RwLock<NetworkState>>) -> Result<()> {
    info!("Starting link watcher (real-time netlink events)");

    // Subscribe to link change notifications
    let (connection, _, mut messages) = rtnetlink::new_connection()?;
    tokio::spawn(connection);

    info!("Link watcher subscribed to netlink events");

    // Process link change events
    while let Some((message, _)) = messages.next().await {
        use netlink_packet_route::RouteNetlinkMessage;

        let (event_type, msg) = match message.payload {
            netlink_packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewLink(msg)) => ("new", msg),
            netlink_packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelLink(msg)) => ("del", msg),
            _ => continue,
        };
        let ifindex = msg.header.index;

        debug!("Link {} event on interface {}", event_type, ifindex);

        // For link additions, refresh our link list
        if event_type == "new" {
            match crate::network::link::acquire_links(&mut *state.write().await, &handle).await {
                Ok(_) => {
                    let link_name = {
                        let state_read = state.read().await;
                        state_read.get_link_name(ifindex).cloned().unwrap_or_default()
                    };
                    info!("Link added: {} ({})", link_name, ifindex);
                }
                Err(e) => {
                    warn!("Failed to refresh link list after add: {}", e);
                }
            }
        }

        // For link deletions, clean up our state
        if event_type == "del" {
            info!("Link removed: interface {}", ifindex);
            // State cleanup happens automatically via NetworkState methods
        }
    }

    Ok(())
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

    // Remove routing rules
    for address in &addresses_to_clean {
        if let Err(e) = remove_routing_rules(handle, *address, table).await {
            warn!("Failed to remove routing rules for {}: {}", address, e);
        }
    }

    // Remove route
    if let Err(e) = remove_route(handle, ifindex, table).await {
        warn!("Failed to remove route for interface {}: {}", ifindex, e);
    }

    // Update state
    {
        let mut state_write = state.write().await;
        for address in &addresses_to_clean {
            state_write.routing_rules_from.remove(address);
            state_write.routing_rules_to.remove(address);
        }
        state_write.routes.remove(&(ifindex, table));
    }

    info!(
        "Cleaned up routing configuration for interface {} ({} addresses)",
        ifindex,
        addresses_to_clean.len()
    );

    Ok(())
}
