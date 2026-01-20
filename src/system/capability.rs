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

    #[test]
    fn test_has_capability() {
        // Test that has_capability doesn't panic
        let result = has_capability(Capability::CAP_NET_ADMIN);
        assert!(result.is_ok());
    }

    #[test]
    fn test_capability_check_multiple() {
        // Check multiple capabilities
        let caps_to_check = vec![
            Capability::CAP_NET_ADMIN,
            Capability::CAP_SYS_ADMIN,
            Capability::CAP_DAC_OVERRIDE,
        ];

        for cap in caps_to_check {
            let result = has_capability(cap);
            assert!(result.is_ok(), "Failed to check capability {:?}", cap);
        }
    }

    #[test]
    fn test_keep_capabilities_toggles() {
        // If running as root, test PR_SET_KEEPCAPS toggling
        if nix::unistd::Uid::effective().is_root() {
            // Enable keep capabilities
            let result = keep_capabilities();
            assert!(result.is_ok(), "Failed to enable keep capabilities");

            // Disable keep capabilities
            let result = clear_keep_capabilities();
            assert!(result.is_ok(), "Failed to disable keep capabilities");
        }
    }

    #[test]
    fn test_apply_capabilities_non_root() {
        // If not running as root, apply_capabilities should handle gracefully
        if !nix::unistd::Uid::effective().is_root() {
            // This will likely fail, but shouldn't panic
            let result = apply_capabilities();
            // Just ensure it returns a result (either Ok or Err)
            let _is_ok = result.is_ok();
        }
    }
}
