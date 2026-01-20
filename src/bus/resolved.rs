// SPDX-License-Identifier: LGPL-3.0-or-later

//! systemd-resolved DBus interface

use anyhow::{Context, Result};
use tracing::{debug, info};
use zbus::Connection;

const RESOLVED_SERVICE: &str = "org.freedesktop.resolve1";
const RESOLVED_PATH: &str = "/org/freedesktop/resolve1";
const RESOLVED_INTERFACE: &str = "org.freedesktop.resolve1.Manager";

/// Set DNS servers for a specific link via systemd-resolved
pub async fn set_link_dns(ifindex: u32, dns_servers: Vec<String>) -> Result<()> {
    if dns_servers.is_empty() {
        debug!("No DNS servers to set for interface {}", ifindex);
        return Ok(());
    }

    info!(
        "Setting DNS servers for interface {}: {:?}",
        ifindex, dns_servers
    );

    let connection = Connection::system()
        .await
        .context("Failed to connect to system bus")?;

    // Convert DNS servers to the format expected by systemd-resolved
    // Format: array of (address_family, address_bytes)
    let dns_array: Vec<(i32, Vec<u8>)> = dns_servers
        .iter()
        .filter_map(|server| {
            if let Ok(addr) = server.parse::<std::net::IpAddr>() {
                match addr {
                    std::net::IpAddr::V4(ipv4) => Some((2, ipv4.octets().to_vec())), // AF_INET = 2
                    std::net::IpAddr::V6(ipv6) => Some((10, ipv6.octets().to_vec())), // AF_INET6 = 10
                }
            } else {
                None
            }
        })
        .collect();

    if dns_array.is_empty() {
        debug!("No valid DNS servers parsed");
        return Ok(());
    }

    // Call SetLinkDNS method
    let proxy = zbus::Proxy::new(
        &connection,
        RESOLVED_SERVICE,
        RESOLVED_PATH,
        RESOLVED_INTERFACE,
    )
    .await
    .context("Failed to create resolved proxy")?;

    proxy
        .call_method("SetLinkDNS", &(ifindex as i32, dns_array))
        .await
        .context("Failed to call SetLinkDNS")?;

    info!("Successfully set DNS servers for interface {}", ifindex);
    Ok(())
}

/// Set DNS domains for a specific link via systemd-resolved
pub async fn set_link_domains(ifindex: u32, domains: Vec<String>) -> Result<()> {
    if domains.is_empty() {
        debug!("No domains to set for interface {}", ifindex);
        return Ok(());
    }

    info!(
        "Setting DNS domains for interface {}: {:?}",
        ifindex, domains
    );

    let connection = Connection::system()
        .await
        .context("Failed to connect to system bus")?;

    // Convert domains to the format expected by systemd-resolved
    // Format: array of (domain, route_only)
    let domain_array: Vec<(String, bool)> = domains.iter().map(|d| (d.clone(), false)).collect();

    // Call SetLinkDomains method
    let proxy = zbus::Proxy::new(
        &connection,
        RESOLVED_SERVICE,
        RESOLVED_PATH,
        RESOLVED_INTERFACE,
    )
    .await
    .context("Failed to create resolved proxy")?;

    proxy
        .call_method("SetLinkDomains", &(ifindex as i32, domain_array))
        .await
        .context("Failed to call SetLinkDomains")?;

    info!("Successfully set DNS domains for interface {}", ifindex);
    Ok(())
}
