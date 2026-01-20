// SPDX-License-Identifier: LGPL-3.0-or-later

//! User and privilege management

use anyhow::{anyhow, Context, Result};
use nix::unistd::{setgid, setuid, Gid, Uid, User};
use tracing::{info, warn};

use super::capability::{apply_capabilities, clear_keep_capabilities, keep_capabilities};

/// Check if running as root
pub fn is_root() -> bool {
    Uid::effective().is_root()
}

/// Lookup user by name and return UID/GID
pub fn lookup_user(username: &str) -> Result<(Uid, Gid)> {
    let user = User::from_name(username)
        .context("Failed to lookup user")?
        .ok_or_else(|| anyhow!("User '{}' not found", username))?;

    Ok((user.uid, user.gid))
}

/// Drop privileges to specified user while retaining necessary capabilities
///
/// This function:
/// 1. Enables PR_SET_KEEPCAPS to retain capabilities across setuid
/// 2. Switches to the target user's UID/GID
/// 3. Disables PR_SET_KEEPCAPS
/// 4. Applies CAP_NET_ADMIN and CAP_SYS_ADMIN capabilities
pub fn drop_privileges(username: &str) -> Result<()> {
    if !is_root() {
        warn!("Not running as root, skipping privilege drop");
        return Ok(());
    }

    info!("Dropping privileges to user '{}'", username);

    // Lookup user
    let (uid, gid) = lookup_user(username)
        .with_context(|| format!("Failed to lookup user '{}'", username))?;

    // Step 1: Enable keeping capabilities across setuid
    keep_capabilities()
        .context("Failed to enable capability retention")?;

    // Step 2: Drop group privileges first (must be done before setuid)
    setgid(gid)
        .with_context(|| format!("Failed to setgid to {}", gid))?;

    // Step 3: Drop user privileges
    setuid(uid)
        .with_context(|| format!("Failed to setuid to {}", uid))?;

    // Step 4: Disable PR_SET_KEEPCAPS
    clear_keep_capabilities()
        .context("Failed to clear capability retention flag")?;

    // Step 5: Apply necessary capabilities
    apply_capabilities()
        .context("Failed to apply capabilities after privilege drop")?;

    info!("Successfully dropped privileges to user '{}' (uid={}, gid={})", username, uid, gid);

    // Verify we're no longer root
    if is_root() {
        return Err(anyhow!("Still running as root after privilege drop!"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_root() {
        // This test will pass or fail depending on how tests are run
        // Just ensure the function works
        let _is_root = is_root();
    }

    #[test]
    fn test_lookup_nonexistent_user() {
        let result = lookup_user("this_user_should_not_exist_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_lookup_root_user() {
        // Root user should always exist on Unix systems
        let result = lookup_user("root");
        assert!(result.is_ok());
        if let Ok((uid, _gid)) = result {
            assert_eq!(uid.as_raw(), 0);
        }
    }
}
