// SPDX-License-Identifier: LGPL-3.0-or-later

//! dhclient lease file parser

use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Lease {
    pub interface: String,
    pub address: String,
    pub subnet_mask: Option<String>,
    pub routers: Vec<String>,
    pub dns_servers: Vec<String>,
    pub domain_name: Option<String>,
    pub hostname: Option<String>,
}

pub fn parse_lease_file(path: &str) -> Result<HashMap<String, Lease>> {
    // TODO: Parse dhclient.leases file
    todo!("Implement lease file parsing")
}
