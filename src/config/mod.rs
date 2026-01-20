// SPDX-License-Identifier: LGPL-3.0-or-later

//! Configuration parsing and management

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

const DEFAULT_CONFIG_PATH: &str = "/etc/netevd/netevd.yaml";
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_BACKEND: &str = "systemd-networkd";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub system: SystemConfig,

    #[serde(default)]
    pub network: NetworkConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SystemConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_backend")]
    pub backend: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkConfig {
    #[serde(default)]
    pub links: String,

    #[serde(default)]
    pub routing_policy_rules: String,

    #[serde(default = "default_true")]
    pub emit_json: bool,

    #[serde(default)]
    pub use_dns: bool,

    #[serde(default)]
    pub use_domain: bool,

    #[serde(default)]
    pub use_hostname: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            backend: DEFAULT_BACKEND.to_string(),
        }
    }
}

impl NetworkConfig {
    /// Get routing policy rule interfaces as a vector
    pub fn get_routing_policy_interfaces(&self) -> Vec<String> {
        self.routing_policy_rules
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            links: String::new(),
            routing_policy_rules: String::new(),
            emit_json: true,
            use_dns: false,
            use_domain: false,
            use_hostname: false,
        }
    }
}

fn default_log_level() -> String {
    DEFAULT_LOG_LEVEL.to_string()
}

fn default_backend() -> String {
    DEFAULT_BACKEND.to_string()
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Parse configuration from file and environment variables
    pub fn parse() -> Result<Self> {
        Self::parse_from_path(DEFAULT_CONFIG_PATH)
    }

    /// Parse configuration from a specific path
    pub fn parse_from_path(path: &str) -> Result<Self> {
        // Try to read config file
        let config = if Path::new(path).exists() {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path))?;

            serde_yaml::from_str(&contents)
                .with_context(|| format!("Failed to parse YAML config: {}", path))?
        } else {
            // Use default config if file doesn't exist
            Config::default()
        };

        // TODO: Override with environment variables if needed
        // Environment variable pattern: NETEVD_SYSTEM_LOG_LEVEL, etc.

        Ok(config)
    }

    /// Get links as a vector
    pub fn get_links(&self) -> Vec<String> {
        self.network.links
            .split_whitespace()
            .map(String::from)
            .collect()
    }

    /// Get routing policy rule links as a vector
    pub fn get_routing_policy_links(&self) -> Vec<String> {
        self.network.routing_policy_rules
            .split_whitespace()
            .map(String::from)
            .collect()
    }

    /// Check if a link should be monitored
    pub fn should_monitor_link(&self, link_name: &str) -> bool {
        let links = self.get_links();
        links.is_empty() || links.contains(&link_name.to_string())
    }

    /// Check if routing policy rules should be configured for a link
    pub fn should_configure_routing_rules(&self, link_name: &str) -> bool {
        let policy_links = self.get_routing_policy_links();
        policy_links.contains(&link_name.to_string())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.system.log_level, "info");
        assert_eq!(config.system.backend, "systemd-networkd");
        assert!(config.network.emit_json);
    }

    #[test]
    fn test_parse_links() {
        let mut config = Config::default();
        config.network.links = "eth0 eth1 wlan0".to_string();

        let links = config.get_links();
        assert_eq!(links, vec!["eth0", "eth1", "wlan0"]);
    }

    #[test]
    fn test_should_monitor_link() {
        let mut config = Config::default();
        config.network.links = "eth0 eth1".to_string();

        assert!(config.should_monitor_link("eth0"));
        assert!(config.should_monitor_link("eth1"));
        assert!(!config.should_monitor_link("wlan0"));
    }

    #[test]
    fn test_should_monitor_all_links_when_empty() {
        let config = Config::default();
        assert!(config.should_monitor_link("eth0"));
        assert!(config.should_monitor_link("wlan0"));
    }
}
