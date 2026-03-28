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

    let connection = get_system_bus().await?;

    let dns_array: Vec<(i32, Vec<u8>)> = dns_servers
        .iter()
        .filter_map(|server| {
            if let Ok(addr) = server.parse::<std::net::IpAddr>() {
                let ifindex_i32 = i32::try_from(ifindex).ok()?;
                let _ = ifindex_i32; // validated
                match addr {
                    std::net::IpAddr::V4(ipv4) => Some((2, ipv4.octets().to_vec())),
                    std::net::IpAddr::V6(ipv6) => Some((10, ipv6.octets().to_vec())),
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

    let ifindex_i32 = i32::try_from(ifindex).context("ifindex out of i32 range")?;

    let proxy = zbus::Proxy::new(
        &connection,
        RESOLVED_SERVICE,
        RESOLVED_PATH,
        RESOLVED_INTERFACE,
    )
    .await
    .context("Failed to create resolved proxy")?;

    proxy
        .call_method("SetLinkDNS", &(ifindex_i32, dns_array))
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

    let connection = get_system_bus().await?;
    let ifindex_i32 = i32::try_from(ifindex).context("ifindex out of i32 range")?;

    let domain_array: Vec<(String, bool)> = domains.iter().map(|d| (d.clone(), false)).collect();

    let proxy = zbus::Proxy::new(
        &connection,
        RESOLVED_SERVICE,
        RESOLVED_PATH,
        RESOLVED_INTERFACE,
    )
    .await
    .context("Failed to create resolved proxy")?;

    proxy
        .call_method("SetLinkDomains", &(ifindex_i32, domain_array))
        .await
        .context("Failed to call SetLinkDomains")?;

    info!("Successfully set DNS domains for interface {}", ifindex);
    Ok(())
}

/// Get or create a cached system bus connection
async fn get_system_bus() -> Result<Connection> {
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    static BUS: OnceLock<Mutex<Option<Connection>>> = OnceLock::new();
    let lock = BUS.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().await;

    if let Some(ref conn) = *guard {
        // Check if connection is still alive by trying a basic operation
        if conn.is_bus() {
            return Ok(conn.clone());
        }
    }

    let conn = Connection::system()
        .await
        .context("Failed to connect to system bus")?;
    *guard = Some(conn.clone());
    Ok(conn)
}
