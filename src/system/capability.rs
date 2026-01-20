// SPDX-License-Identifier: LGPL-3.0-or-later

//! Linux capability management

use anyhow::{Context, Result};
use caps::{CapSet, Capability};
use nix::sys::prctl;
use std::collections::HashSet;

/// Apply necessary capabilities (CAP_NET_ADMIN and CAP_SYS_ADMIN)
/// This should be called after privilege dropping
pub fn apply_capabilities() -> Result<()> {
    // Create a set of capabilities we need
    let mut capabilities = HashSet::new();
    capabilities.insert(Capability::CAP_NET_ADMIN);
    capabilities.insert(Capability::CAP_SYS_ADMIN);

    // Set in permitted set
    caps::set(None, CapSet::Permitted, &capabilities)
        .context("Failed to set capabilities in permitted set")?;

    // Set in effective set
    caps::set(None, CapSet::Effective, &capabilities)
        .context("Failed to set capabilities in effective set")?;

    // Set in inheritable set
    caps::set(None, CapSet::Inheritable, &capabilities)
        .context("Failed to set capabilities in inheritable set")?;

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
