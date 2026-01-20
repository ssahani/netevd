// SPDX-License-Identifier: LGPL-3.0-or-later

//! Route management

use anyhow::{Context, Result};
use futures::stream::TryStreamExt;
use netlink_packet_route::route::{RouteAddress, RouteAttribute, RouteMessage};
use rtnetlink::Handle;
use std::net::IpAddr;
use tracing::{debug, info, warn};

use super::routing_rule::ROUTE_TABLE_BASE;

/// Discover the default gateway for a specific interface
pub async fn discover_gateway(handle: &Handle, ifindex: u32) -> Result<Option<IpAddr>> {
    let mut routes = handle.route().get(rtnetlink::IpVersion::V4).execute();

    while let Some(route) = routes
        .try_next()
        .await
        .context("Failed to get next route")?
    {
        // Look for default route (0.0.0.0/0) on this interface
        if is_default_route(&route) && route_matches_interface(&route, ifindex) {
            if let Some(gateway) = extract_gateway(&route) {
                // RouteAddress is a type alias or enum - convert to IpAddr
                // RouteAddress likely has Inet(Ipv4Addr) or Inet6(Ipv6Addr) variants
                match gateway {
                    RouteAddress::Inet(ipv4) => {
                        debug!("Found gateway {} for interface {}", ipv4, ifindex);
                        return Ok(Some(IpAddr::V4(ipv4)));
                    }
                    RouteAddress::Inet6(ipv6) => {
                        debug!("Found gateway {} for interface {}", ipv6, ifindex);
                        return Ok(Some(IpAddr::V6(ipv6)));
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(None)
}

/// Add a default route via gateway to a custom routing table
pub async fn add_route(
    handle: &Handle,
    ifindex: u32,
    gateway: IpAddr,
    table: u32,
) -> Result<()> {
    info!(
        "Adding route: ifindex={}, gateway={}, table={}",
        ifindex, gateway, table
    );

    // Convert gateway to the proper type
    match gateway {
        IpAddr::V4(gw_v4) => {
            handle
                .route()
                .add()
                .v4()
                .destination_prefix(std::net::Ipv4Addr::new(0, 0, 0, 0), 0) // 0.0.0.0/0
                .gateway(gw_v4)
                .output_interface(ifindex)
                .table_id(table)
                .execute()
                .await
                .with_context(|| {
                    format!(
                        "Failed to add IPv4 route for interface {} via {} in table {}",
                        ifindex, gateway, table
                    )
                })?;
        }
        IpAddr::V6(gw_v6) => {
            handle
                .route()
                .add()
                .v6()
                .destination_prefix(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)
                .gateway(gw_v6)
                .output_interface(ifindex)
                .table_id(table)
                .execute()
                .await
                .with_context(|| {
                    format!(
                        "Failed to add IPv6 route for interface {} via {} in table {}",
                        ifindex, gateway, table
                    )
                })?;
        }
    }

    info!("Successfully added route in table {}", table);
    Ok(())
}

/// Remove a route from a custom routing table
pub async fn remove_route(handle: &Handle, ifindex: u32, table: u32) -> Result<()> {
    info!("Removing routes for ifindex={} in table={}", ifindex, table);

    // Get all routes in the specified table
    let mut routes = handle.route().get(rtnetlink::IpVersion::V4).execute();

    let mut removed = false;
    while let Some(route) = routes
        .try_next()
        .await
        .context("Failed to get next route")?
    {
        if route_in_table(&route, table) && route_matches_interface(&route, ifindex) {
            // Delete this route
            if let Err(e) = handle.route().del(route).execute().await {
                warn!("Failed to delete route: {}", e);
            } else {
                debug!("Deleted route in table {}", table);
                removed = true;
            }
        }
    }

    if removed {
        info!("Successfully removed routes from table {}", table);
    }

    Ok(())
}

/// Calculate the custom routing table number for an interface
pub fn calculate_table_id(ifindex: u32) -> u32 {
    ROUTE_TABLE_BASE + ifindex
}

/// Check if a route is a default route (0.0.0.0/0)
fn is_default_route(route: &RouteMessage) -> bool {
    // Default route has destination length of 0
    route.header.destination_prefix_length == 0
}

/// Check if a route matches a specific interface
fn route_matches_interface(route: &RouteMessage, ifindex: u32) -> bool {
    route.attributes.iter().any(|attr| {
        matches!(
            attr,
            RouteAttribute::Oif(idx) if *idx == ifindex
        )
    })
}

/// Check if a route is in a specific table
fn route_in_table(route: &RouteMessage, table: u32) -> bool {
    // Check the table attribute
    if let Some(RouteAttribute::Table(t)) = route
        .attributes
        .iter()
        .find(|a| matches!(a, RouteAttribute::Table(_)))
    {
        return *t == table;
    }

    // Also check the header table field for small table IDs
    route.header.table as u32 == table
}

/// Extract gateway RouteAddress from a route
fn extract_gateway(route: &RouteMessage) -> Option<RouteAddress> {
    for attr in &route.attributes {
        if let RouteAttribute::Gateway(gw_addr) = attr {
            return Some(gw_addr.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_table_id() {
        assert_eq!(calculate_table_id(2), ROUTE_TABLE_BASE + 2);
        assert_eq!(calculate_table_id(10), ROUTE_TABLE_BASE + 10);
    }
}
