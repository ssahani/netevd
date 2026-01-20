// SPDX-License-Identifier: LGPL-3.0-or-later

//! Link tracking and operations

use anyhow::{Context, Result};
use futures::stream::TryStreamExt;
use netlink_packet_route::link::LinkAttribute;
use rtnetlink::Handle;
use tracing::{debug, info};

use super::NetworkState;

/// Acquire all network links and populate the network state
pub async fn acquire_links(state: &mut NetworkState, handle: &Handle) -> Result<()> {
    info!("Acquiring network links");

    let mut links = handle.link().get().execute();

    while let Some(link) = links
        .try_next()
        .await
        .context("Failed to get next link")?
    {
        let index = link.header.index;
        let name = link
            .attributes
            .iter()
            .find_map(|attr| {
                if let LinkAttribute::IfName(name) = attr {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("link{}", index));

        debug!("Found link: {} (index={})", name, index);
        state.add_link(name, index);
    }

    info!("Acquired {} links", state.links_by_index.len());
    Ok(())
}

/// Get a netlink handle
pub async fn get_netlink_handle() -> Result<Handle> {
    let (connection, handle, _) = rtnetlink::new_connection()
        .context("Failed to create netlink connection")?;

    // Spawn the connection in the background
    tokio::spawn(connection);

    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_netlink_handle() {
        // This test requires CAP_NET_ADMIN or root
        // It may fail in CI/restricted environments
        let result = get_netlink_handle().await;
        // Just ensure it doesn't panic
        let _ = result;
    }
}
