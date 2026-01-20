// SPDX-License-Identifier: LGPL-3.0-or-later

//! dhclient file watcher

pub mod parser;

use anyhow::Result;
use rtnetlink::Handle;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::network::NetworkState;

pub async fn watch_lease_file(
    _config: Config,
    _handle: Handle,
    _state: Arc<RwLock<NetworkState>>,
) -> Result<()> {
    // TODO: Watch /var/lib/dhclient/dhclient.leases using notify crate
    todo!("Implement dhclient lease file watcher")
}
