// SPDX-License-Identifier: LGPL-3.0-or-later

//! Routing policy rule management

use anyhow::{Context, Result};
use futures::stream::TryStreamExt;
use netlink_packet_route::rule::{RuleAction, RuleAttribute, RuleMessage};
use rtnetlink::Handle;
use std::net::IpAddr;
use tracing::{debug, info, warn};

/// Base table number for custom routing tables
/// Using an uncommon number (200) to avoid conflicts with existing configurations
/// Linux reserves tables 0-255 for system use, but 200-250 are rarely used
pub const ROUTE_TABLE_BASE: u32 = 200;

/// Add a routing policy rule (from address -> table)
pub async fn add_routing_rule_from(
    handle: &Handle,
    address: IpAddr,
    table: u32,
) -> Result<()> {
    info!(
        "Adding routing rule: from {} table {}",
        address, table
    );

    match address {
        IpAddr::V4(ipv4) => {
            handle
                .rule()
                .add()
                .v4()
                .source_prefix(ipv4, 32)
                .table_id(table)
                .action(RuleAction::ToTable)
                .execute()
                .await
                .with_context(|| {
                    format!("Failed to add IPv4 routing rule: from {} table {}", address, table)
                })?;
        }
        IpAddr::V6(ipv6) => {
            handle
                .rule()
                .add()
                .v6()
                .source_prefix(ipv6, 128)
                .table_id(table)
                .action(RuleAction::ToTable)
                .execute()
                .await
                .with_context(|| {
                    format!("Failed to add IPv6 routing rule: from {} table {}", address, table)
                })?;
        }
    }

    info!("Successfully added 'from' routing rule");
    Ok(())
}

/// Add a routing policy rule (to address -> table)
pub async fn add_routing_rule_to(
    handle: &Handle,
    address: IpAddr,
    table: u32,
) -> Result<()> {
    info!(
        "Adding routing rule: to {} table {}",
        address, table
    );

    match address {
        IpAddr::V4(ipv4) => {
            handle
                .rule()
                .add()
                .v4()
                .destination_prefix(ipv4, 32)
                .table_id(table)
                .action(RuleAction::ToTable)
                .execute()
                .await
                .with_context(|| {
                    format!("Failed to add IPv4 routing rule: to {} table {}", address, table)
                })?;
        }
        IpAddr::V6(ipv6) => {
            handle
                .rule()
                .add()
                .v6()
                .destination_prefix(ipv6, 128)
                .table_id(table)
                .action(RuleAction::ToTable)
                .execute()
                .await
                .with_context(|| {
                    format!("Failed to add IPv6 routing rule: to {} table {}", address, table)
                })?;
        }
    }

    info!("Successfully added 'to' routing rule");
    Ok(())
}

/// Remove routing policy rules for an address
pub async fn remove_routing_rules(
    handle: &Handle,
    address: IpAddr,
    table: u32,
) -> Result<()> {
    info!(
        "Removing routing rules for address {} in table {}",
        address, table
    );

    let ip_version = match address {
        IpAddr::V4(_) => rtnetlink::IpVersion::V4,
        IpAddr::V6(_) => rtnetlink::IpVersion::V6,
    };

    let mut rules = handle.rule().get(ip_version).execute();

    let mut removed_count = 0;
    while let Some(rule) = rules
        .try_next()
        .await
        .context("Failed to get next rule")?
    {
        if rule_matches(&rule, &address, table) {
            // Delete this rule
            if let Err(e) = handle.rule().del(rule).execute().await {
                warn!("Failed to delete routing rule: {}", e);
            } else {
                debug!("Deleted routing rule for address {}", address);
                removed_count += 1;
            }
        }
    }

    if removed_count > 0 {
        info!("Successfully removed {} routing rules", removed_count);
    }

    Ok(())
}

/// Check if a rule matches the given address and table
fn rule_matches(rule: &RuleMessage, address: &IpAddr, table: u32) -> bool {
    // Check if the rule's table matches
    let rule_table = rule.attributes.iter().find_map(|attr| {
        if let RuleAttribute::Table(t) = attr {
            Some(*t)
        } else {
            None
        }
    });

    if rule_table != Some(table) && rule.header.table as u32 != table {
        return false;
    }

    // Check if the source or destination matches our address
    rule.attributes.iter().any(|attr| match attr {
        RuleAttribute::Source(src) => src == address,
        RuleAttribute::Destination(dst) => dst == address,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_table_base() {
        assert_eq!(ROUTE_TABLE_BASE, 200);
    }

    #[test]
    fn test_table_calculation() {
        // From route.rs
        assert_eq!(ROUTE_TABLE_BASE + 2, 202);
        assert_eq!(ROUTE_TABLE_BASE + 10, 210);
    }
}
