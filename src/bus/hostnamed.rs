// SPDX-License-Identifier: LGPL-3.0-or-later

//! systemd-hostnamed DBus interface

use anyhow::{Context, Result};
use tracing::info;
use zbus::Connection;

const HOSTNAMED_SERVICE: &str = "org.freedesktop.hostname1";
const HOSTNAMED_PATH: &str = "/org/freedesktop/hostname1";
const HOSTNAMED_INTERFACE: &str = "org.freedesktop.hostname1";

/// Set static hostname via systemd-hostnamed
pub async fn set_static_hostname(hostname: &str) -> Result<()> {
    if hostname.is_empty() {
        return Ok(());
    }

    info!("Setting static hostname to: {}", hostname);

    let connection = get_system_bus().await?;

    let proxy = zbus::Proxy::new(
        &connection,
        HOSTNAMED_SERVICE,
        HOSTNAMED_PATH,
        HOSTNAMED_INTERFACE,
    )
    .await
    .context("Failed to create hostnamed proxy")?;

    proxy
        .call_method("SetStaticHostname", &(hostname, false))
        .await
        .context("Failed to call SetStaticHostname")?;

    info!("Successfully set static hostname to: {}", hostname);
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
