// SPDX-License-Identifier: LGPL-3.0-or-later

//! dhclient lease file parser

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Default)]
pub struct Lease {
    pub interface: String,
    pub address: String,
    pub subnet_mask: Option<String>,
    pub routers: Vec<String>,
    pub dns_servers: Vec<String>,
    pub domain_name: Option<String>,
    pub hostname: Option<String>,
}

/// Parse dhclient lease file and return map of interface -> lease
pub fn parse_lease_file(path: &str) -> Result<HashMap<String, Lease>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read lease file: {}", path))?;

    let mut leases: HashMap<String, Lease> = HashMap::new();
    let mut current_lease: Option<Lease> = None;
    let mut in_lease_block = false;

    for line in contents.lines() {
        let line = line.trim();

        // Start of lease block
        if line.starts_with("lease") && line.contains('{') {
            in_lease_block = true;
            let mut lease = Lease::default();

            // Extract IP address from "lease 192.168.1.100 {"
            if let Some(addr) = line
                .strip_prefix("lease")
                .and_then(|s| s.trim().strip_suffix('{'))
            {
                lease.address = addr.trim().to_string();
            }

            current_lease = Some(lease);
            continue;
        }

        // End of lease block
        if line == "}" && in_lease_block {
            in_lease_block = false;
            if let Some(lease) = current_lease.take() {
                if !lease.interface.is_empty() && !lease.address.is_empty() {
                    leases.insert(lease.interface.clone(), lease);
                }
            }
            continue;
        }

        // Parse lease properties
        if in_lease_block {
            if let Some(lease) = current_lease.as_mut() {
                // interface "eth0";
                if line.starts_with("interface") {
                    if let Some(iface) = extract_quoted_value(line) {
                        lease.interface = iface;
                    }
                }
                // fixed-address 192.168.1.100;
                else if line.starts_with("fixed-address") {
                    if let Some(addr) = extract_value(line) {
                        lease.address = addr;
                    }
                }
                // option subnet-mask 255.255.255.0;
                else if line.contains("subnet-mask") {
                    if let Some(mask) = extract_value(line) {
                        lease.subnet_mask = Some(mask);
                    }
                }
                // option routers 192.168.1.1;
                else if line.contains("routers") {
                    if let Some(router) = extract_value(line) {
                        lease.routers = router.split(',').map(|s| s.trim().to_string()).collect();
                    }
                }
                // option domain-name-servers 8.8.8.8, 8.8.4.4;
                else if line.contains("domain-name-servers") {
                    if let Some(dns) = extract_value(line) {
                        lease.dns_servers = dns.split(',').map(|s| s.trim().to_string()).collect();
                    }
                }
                // option domain-name "example.com";
                else if line.contains("domain-name") && !line.contains("domain-name-servers") {
                    if let Some(domain) = extract_quoted_value(line) {
                        lease.domain_name = Some(domain);
                    }
                }
                // option host-name "myhost";
                else if line.contains("host-name") {
                    if let Some(hostname) = extract_quoted_value(line) {
                        lease.hostname = Some(hostname);
                    }
                }
            }
        }
    }

    Ok(leases)
}

/// Extract value from lines like "option routers 192.168.1.1;"
fn extract_value(line: &str) -> Option<String> {
    line.split_whitespace()
        .last()
        .and_then(|s| s.strip_suffix(';'))
        .map(|s| s.to_string())
}

/// Extract quoted value from lines like 'interface "eth0";'
fn extract_quoted_value(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line.rfind('"')?;
    if start < end {
        Some(line[start + 1..end].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_value() {
        assert_eq!(
            extract_value("option routers 192.168.1.1;"),
            Some("192.168.1.1".to_string())
        );
        assert_eq!(
            extract_value("fixed-address 10.0.0.5;"),
            Some("10.0.0.5".to_string())
        );
    }

    #[test]
    fn test_extract_quoted_value() {
        assert_eq!(
            extract_quoted_value("interface \"eth0\";"),
            Some("eth0".to_string())
        );
        assert_eq!(
            extract_quoted_value("option domain-name \"example.com\";"),
            Some("example.com".to_string())
        );
    }
}
