// SPDX-License-Identifier: LGPL-3.0-or-later

//! Linux capability management

use anyhow::{Context, Result};
use caps::{CapSet, Capability};
use nix::sys::prctl;
use std::collections::HashSet;

/// Apply necessary capabilities for network operations
///
/// Only CAP_NET_ADMIN is needed for:
/// - Managing routing tables and policy rules
/// - Configuring network interfaces
/// - Accessing netlink sockets
///
/// NOTE: CAP_SYS_ADMIN has been removed as it's overly broad and provides
/// unnecessary privileges. CAP_NET_ADMIN is sufficient for all network
/// configuration operations this daemon performs.
///
/// This should be called after privilege dropping.
pub fn apply_capabilities() -> Result<()> {
    // Only request CAP_NET_ADMIN - sufficient for all network operations
    let mut capabilities = HashSet::new();
    capabilities.insert(Capability::CAP_NET_ADMIN);

    // Set in permitted set (capability pool we can draw from)
    caps::set(None, CapSet::Permitted, &capabilities)
        .context("Failed to set capabilities in permitted set")?;

    // Set in effective set (actually active capabilities)
    caps::set(None, CapSet::Effective, &capabilities)
        .context("Failed to set capabilities in effective set")?;

    // Do NOT set Inheritable - child processes (scripts) should not
    // inherit network admin capabilities for security

    tracing::info!("Applied capabilities: {:?}", capabilities);

    // Verify the capability was actually acquired
    let effective = caps::read(None, CapSet::Effective)
        .context("Failed to read effective capabilities")?;

    if !effective.contains(&Capability::CAP_NET_ADMIN) {
        anyhow::bail!("Failed to acquire CAP_NET_ADMIN capability");
    }

    Ok(())
}

/// Enable keeping capabilities across setuid
pub fn keep_capabilities() -> Result<()> {
    prctl::set_keepcaps(true)
        .context("Failed to set PR_SET_KEEPCAPS to true")?;
    Ok(())
}

/// Disable keeping capabilities (should be called after setuid)
pub fn clear_keep_capabilities() -> Result<()> {
    prctl::set_keepcaps(false)
        .context("Failed to set PR_SET_KEEPCAPS to false")?;
    Ok(())
}

/// Check if we have a specific capability
pub fn has_capability(cap: Capability) -> Result<bool> {
    caps::has_cap(None, CapSet::Effective, cap)
        .with_context(|| format!("Failed to check for capability {:?}", cap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_enum() {
        // Just ensure the capability enums work
        let _net_admin = Capability::CAP_NET_ADMIN;
        let _sys_admin = Capability::CAP_SYS_ADMIN;
    }
}
