use anyhow::Result;
use rtnetlink::Handle;
use std::net::Ipv6Addr;
use tracing::{debug, error, info};

/// IPv6 Policy Routing Support
///
/// This module handles IPv6-specific routing policy rules and source address selection

const IPV6_RULE_PRIORITY_BASE: u32 = 32765;

/// Add IPv6 routing policy rule for source-based routing
/// Note: Currently a placeholder - full implementation requires rtnetlink API updates
pub async fn add_ipv6_routing_rule(
    _handle: &Handle,
    address: &Ipv6Addr,
    table_id: u32,
) -> Result<()> {
    info!(
        "Adding IPv6 routing policy rule: from {} lookup table {}",
        address, table_id
    );

    // TODO: Implement when rtnetlink supports IPv6 rules properly
    // For now, this would need to be done via `ip -6 rule add` command
    debug!("IPv6 routing policy rules (placeholder for future implementation)");
    Ok(())
}

/// Remove IPv6 routing policy rule
/// Note: Currently a placeholder - full implementation requires rtnetlink API updates
pub async fn remove_ipv6_routing_rule(
    _handle: &Handle,
    address: &Ipv6Addr,
    table_id: u32,
) -> Result<()> {
    info!(
        "Removing IPv6 routing policy rule: from {} lookup table {}",
        address, table_id
    );

    // TODO: Implement when rtnetlink supports IPv6 rules properly
    debug!("IPv6 routing policy rules removal (placeholder for future implementation)");
    Ok(())
}

/// Add IPv6 default route in custom table
/// Note: Currently a placeholder - full implementation requires rtnetlink API updates
pub async fn add_ipv6_default_route(
    _handle: &Handle,
    gateway: Ipv6Addr,
    ifindex: u32,
    table_id: u32,
) -> Result<()> {
    info!(
        "Adding IPv6 default route: via {} dev {} table {}",
        gateway, ifindex, table_id
    );

    // TODO: Implement when rtnetlink supports IPv6 routes properly
    debug!("IPv6 default route (placeholder for future implementation)");
    Ok(())
}

/// Remove IPv6 default route from custom table
/// Note: Currently a placeholder - full implementation requires rtnetlink API updates
pub async fn remove_ipv6_default_route(
    _handle: &Handle,
    gateway: Ipv6Addr,
    ifindex: u32,
    table_id: u32,
) -> Result<()> {
    info!(
        "Removing IPv6 default route: via {} dev {} table {}",
        gateway, ifindex, table_id
    );

    // TODO: Implement when rtnetlink supports IPv6 routes properly
    Ok(())
}

/// Check if an IPv6 address is link-local
pub fn is_link_local(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

/// Check if an IPv6 address is unique local (ULA)
pub fn is_unique_local(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

/// Check if an IPv6 address is global unicast
pub fn is_global_unicast(addr: &Ipv6Addr) -> bool {
    !addr.is_loopback()
        && !addr.is_multicast()
        && !is_link_local(addr)
        && !is_unique_local(addr)
        && !addr.is_unspecified()
}

/// Get preferred source address for IPv6 (RFC 6724)
pub fn select_source_address(addresses: &[Ipv6Addr]) -> Option<Ipv6Addr> {
    // Prefer global unicast addresses
    if let Some(&addr) = addresses.iter().find(|a| is_global_unicast(a)) {
        return Some(addr);
    }

    // Then unique local addresses
    if let Some(&addr) = addresses.iter().find(|a| is_unique_local(a)) {
        return Some(addr);
    }

    // Finally link-local addresses
    if let Some(&addr) = addresses.iter().find(|a| is_link_local(a)) {
        return Some(addr);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_is_link_local() {
        let addr = Ipv6Addr::from_str("fe80::1").unwrap();
        assert!(is_link_local(&addr));

        let addr = Ipv6Addr::from_str("2001:db8::1").unwrap();
        assert!(!is_link_local(&addr));
    }

    #[test]
    fn test_is_unique_local() {
        let addr = Ipv6Addr::from_str("fc00::1").unwrap();
        assert!(is_unique_local(&addr));

        let addr = Ipv6Addr::from_str("fd00::1").unwrap();
        assert!(is_unique_local(&addr));

        let addr = Ipv6Addr::from_str("2001:db8::1").unwrap();
        assert!(!is_unique_local(&addr));
    }

    #[test]
    fn test_is_global_unicast() {
        let addr = Ipv6Addr::from_str("2001:db8::1").unwrap();
        assert!(is_global_unicast(&addr));

        let addr = Ipv6Addr::from_str("fe80::1").unwrap();
        assert!(!is_global_unicast(&addr));

        let addr = Ipv6Addr::from_str("fc00::1").unwrap();
        assert!(!is_global_unicast(&addr));
    }

    #[test]
    fn test_select_source_address() {
        let addresses = vec![
            Ipv6Addr::from_str("fe80::1").unwrap(),
            Ipv6Addr::from_str("2001:db8::1").unwrap(),
            Ipv6Addr::from_str("fc00::1").unwrap(),
        ];

        let selected = select_source_address(&addresses);
        assert_eq!(selected, Some(Ipv6Addr::from_str("2001:db8::1").unwrap()));
    }
}
