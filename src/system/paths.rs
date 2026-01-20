// SPDX-License-Identifier: LGPL-3.0-or-later

//! File and path utilities

use std::path::PathBuf;

pub const CONFIG_DIR: &str = "/etc/netevd";
pub const CONFIG_FILE: &str = "/etc/netevd/netevd.yaml";
pub const DHCLIENT_LEASE_PATH: &str = "/var/lib/dhclient/dhclient.leases";
pub const SYSTEMD_NETIF_LINKS: &str = "/run/systemd/netif/links";
pub const SYSTEMD_NETIF_STATE: &str = "/run/systemd/netif/state";

pub fn get_script_dir(state: &str) -> String {
    PathBuf::from(CONFIG_DIR)
        .join(format!("{}.d", state))
        .to_string_lossy()
        .to_string()
}
