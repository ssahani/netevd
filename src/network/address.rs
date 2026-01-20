// SPDX-License-Identifier: LGPL-3.0-or-later

//! Address operations

use anyhow::{Context, Result};
use futures::stream::TryStreamExt;
use netlink_packet_route::address::AddressAttribute;
use netlink_packet_route::AddressFamily;
use rtnetlink::Handle;
use std::net::IpAddr;
use tracing::debug;

/// Get all IPv4 addresses for a specific interface
pub async fn get_ipv4_addresses(handle: &Handle, ifindex: u32) -> Result<Vec<IpAddr>> {
    let mut addresses = Vec::new();
    let mut addr_stream = handle.address().get().set_link_index_filter(ifindex).execute();

    while let Some(msg) = addr_stream
        .try_next()
        .await
        .context("Failed to get next address")?
    {
        // Only process IPv4 addresses
        if msg.header.family != AddressFamily::Inet {
            continue;
        }

        // Extract the address from attributes
        for attr in msg.attributes {
            if let AddressAttribute::Address(ip_addr) = attr {
                // Skip link-local addresses (169.254.0.0/16)
                if !is_link_local(&ip_addr) {
                    debug!("Found IPv4 address {} on interface {}", ip_addr, ifindex);
                    addresses.push(ip_addr);
                }
            }
        }
    }

    Ok(addresses)
}

/// Get all addresses (IPv4 and IPv6) for a specific interface
pub async fn get_all_addresses(handle: &Handle, ifindex: u32) -> Result<Vec<IpAddr>> {
    let mut addresses = Vec::new();
    let mut addr_stream = handle.address().get().set_link_index_filter(ifindex).execute();

    while let Some(msg) = addr_stream
        .try_next()
        .await
        .context("Failed to get next address")?
    {
        // Extract the address from attributes
        for attr in msg.attributes {
            if let AddressAttribute::Address(ip_addr) = attr {
                // Skip link-local addresses
                if !is_link_local(&ip_addr) {
                    debug!("Found address {} on interface {}", ip_addr, ifindex);
                    addresses.push(ip_addr);
                }
            }
        }
    }

    Ok(addresses)
}

/// Check if an IP address is link-local
/// IPv4: 169.254.0.0/16
/// IPv6: fe80::/10
fn is_link_local(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            octets[0] == 169 && octets[1] == 254
        }
        IpAddr::V6(ipv6) => {
            // fe80::/10 - first 10 bits are 1111111010
            let segments = ipv6.segments();
            (segments[0] & 0xffc0) == 0xfe80
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_is_link_local_ipv4() {
        assert!(is_link_local(&IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));
        assert!(is_link_local(&IpAddr::V4(Ipv4Addr::new(169, 254, 255, 255))));
        assert!(!is_link_local(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(!is_link_local(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn test_is_link_local_ipv6() {
        assert!(is_link_local(&IpAddr::V6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        ))));
        assert!(!is_link_local(&IpAddr::V6(Ipv6Addr::new(
            0x2001, 0xdb8, 0, 0, 0, 0, 0, 1
        ))));
    }
}
