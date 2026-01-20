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

/// Extract value from lines like "option routers 192.168.1.1;" or "option domain-name-servers 8.8.8.8, 8.8.4.4;"
fn extract_value(line: &str) -> Option<String> {
    // Find the last occurrence of a space before the semicolon
    // This handles both single values and comma-separated lists
    let semicolon_pos = line.rfind(';')?;
    let value_part = &line[..semicolon_pos];

    // Find where the actual value starts (after the last space before option name ends)
    // For "  option domain-name-servers 8.8.8.8, 8.8.4.4", we want "8.8.8.8, 8.8.4.4"
    let parts: Vec<&str> = value_part.split_whitespace().collect();
    if parts.len() >= 3 {
        // Join everything after "option" and the option name
        Some(parts[2..].join(" "))
    } else if parts.len() == 2 {
        // Simple case like "fixed-address 192.168.1.1"
        Some(parts[1].to_string())
    } else {
        None
    }
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

    #[test]
    fn test_parse_valid_lease_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lease_content = r#"
lease 192.168.1.100 {
  interface "eth0";
  fixed-address 192.168.1.100;
  option subnet-mask 255.255.255.0;
  option routers 192.168.1.1;
  option domain-name-servers 8.8.8.8, 8.8.4.4;
  option domain-name "example.com";
  option host-name "myhost";
}
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(lease_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let leases = parse_lease_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(leases.len(), 1);
        let lease = leases.get("eth0").unwrap();
        assert_eq!(lease.address, "192.168.1.100");
        assert_eq!(lease.subnet_mask, Some("255.255.255.0".to_string()));
        assert_eq!(lease.routers, vec!["192.168.1.1"]);
        assert_eq!(lease.dns_servers, vec!["8.8.8.8", "8.8.4.4"]);
        assert_eq!(lease.domain_name, Some("example.com".to_string()));
        assert_eq!(lease.hostname, Some("myhost".to_string()));
    }

    #[test]
    fn test_parse_multiple_leases() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lease_content = r#"
lease 192.168.1.100 {
  interface "eth0";
  option routers 192.168.1.1;
}

lease 10.0.0.50 {
  interface "eth1";
  option routers 10.0.0.1;
}
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(lease_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let leases = parse_lease_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(leases.len(), 2);
        assert!(leases.contains_key("eth0"));
        assert!(leases.contains_key("eth1"));
    }

    #[test]
    fn test_parse_malformed_lease_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Missing interface - should be ignored
        let lease_content = r#"
lease 192.168.1.100 {
  option routers 192.168.1.1;
}
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(lease_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let leases = parse_lease_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should be empty because interface is missing
        assert_eq!(leases.len(), 0);
    }

    #[test]
    fn test_parse_empty_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"").unwrap();
        temp_file.flush().unwrap();

        let leases = parse_lease_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(leases.len(), 0);
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_lease_file("/nonexistent/path/to/lease.file");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_value_edge_cases() {
        assert_eq!(extract_value("no semicolon"), None);
        assert_eq!(extract_value(""), None);
        assert_eq!(extract_value(";"), None); // Changed: empty line with just semicolon has no value
    }

    #[test]
    fn test_extract_quoted_value_edge_cases() {
        assert_eq!(extract_quoted_value("no quotes"), None);
        assert_eq!(extract_quoted_value("\"only one quote"), None);
        assert_eq!(extract_quoted_value("\"\""), Some("".to_string()));
    }

    #[test]
    fn test_parse_lease_with_comments() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lease_content = r#"
# Comment line
lease 192.168.1.100 {
  interface "eth0";  # inline comment
  option routers 192.168.1.1;
}
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(lease_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let leases = parse_lease_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(leases.len(), 1);
        assert!(leases.contains_key("eth0"));
    }

    #[test]
    fn test_parse_lease_multiple_dns() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lease_content = r#"
lease 192.168.1.100 {
  interface "eth0";
  option domain-name-servers 8.8.8.8, 8.8.4.4, 1.1.1.1;
}
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(lease_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let leases = parse_lease_file(temp_file.path().to_str().unwrap()).unwrap();

        let lease = leases.get("eth0").unwrap();
        assert_eq!(lease.dns_servers.len(), 3);
        assert_eq!(lease.dns_servers, vec!["8.8.8.8", "8.8.4.4", "1.1.1.1"]);
    }
}
