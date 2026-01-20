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

    let connection = Connection::system()
        .await
        .context("Failed to connect to system bus")?;

    // Call SetStaticHostname method
    let proxy = zbus::Proxy::new(
        &connection,
        HOSTNAMED_SERVICE,
        HOSTNAMED_PATH,
        HOSTNAMED_INTERFACE,
    )
    .await
    .context("Failed to create hostnamed proxy")?;

    // SetStaticHostname(hostname: String, interactive: bool)
    proxy
        .call_method("SetStaticHostname", &(hostname, false))
        .await
        .context("Failed to call SetStaticHostname")?;

    info!("Successfully set static hostname to: {}", hostname);
    Ok(())
}
