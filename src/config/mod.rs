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
    pub monitoring: MonitoringConfig,

    #[serde(default)]
    pub routing: RoutingConfig,

    #[serde(default)]
    pub backends: BackendsConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SystemConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_backend")]
    pub backend: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct MonitoringConfig {
    #[serde(default)]
    pub interfaces: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RoutingConfig {
    #[serde(default)]
    pub policy_rules: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct BackendsConfig {
    #[serde(default)]
    pub systemd_networkd: SystemdNetworkdConfig,

    #[serde(default)]
    pub dhclient: DhclientConfig,

    #[serde(default)]
    pub networkmanager: NetworkManagerConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SystemdNetworkdConfig {
    #[serde(default = "default_true")]
    pub emit_json: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DhclientConfig {
    #[serde(default)]
    pub use_dns: bool,

    #[serde(default)]
    pub use_domain: bool,

    #[serde(default)]
    pub use_hostname: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct NetworkManagerConfig {
    // Placeholder for future NetworkManager-specific options
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            backend: DEFAULT_BACKEND.to_string(),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
        }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            policy_rules: Vec::new(),
        }
    }
}

impl Default for BackendsConfig {
    fn default() -> Self {
        Self {
            systemd_networkd: SystemdNetworkdConfig::default(),
            dhclient: DhclientConfig::default(),
            networkmanager: NetworkManagerConfig::default(),
        }
    }
}

impl Default for SystemdNetworkdConfig {
    fn default() -> Self {
        Self { emit_json: true }
    }
}

impl Default for DhclientConfig {
    fn default() -> Self {
        Self {
            use_dns: false,
            use_domain: false,
            use_hostname: false,
        }
    }
}

impl Default for NetworkManagerConfig {
    fn default() -> Self {
        Self {}
    }
}

impl MonitoringConfig {
    /// Get interfaces as a vector
    pub fn get_interfaces(&self) -> Vec<String> {
        self.interfaces.clone()
    }
}

impl RoutingConfig {
    /// Get routing policy rule interfaces as a vector
    pub fn get_routing_policy_interfaces(&self) -> Vec<String> {
        self.policy_rules.clone()
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
        self.monitoring.interfaces.clone()
    }

    /// Get routing policy rule links as a vector
    pub fn get_routing_policy_links(&self) -> Vec<String> {
        self.routing.policy_rules.clone()
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

    /// Get emit_json setting
    pub fn get_emit_json(&self) -> bool {
        self.backends.systemd_networkd.emit_json
    }

    /// Get use_dns setting
    pub fn get_use_dns(&self) -> bool {
        self.backends.dhclient.use_dns
    }

    /// Get use_domain setting
    pub fn get_use_domain(&self) -> bool {
        self.backends.dhclient.use_domain
    }

    /// Get use_hostname setting
    pub fn get_use_hostname(&self) -> bool {
        self.backends.dhclient.use_hostname
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            monitoring: MonitoringConfig::default(),
            routing: RoutingConfig::default(),
            backends: BackendsConfig::default(),
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
        assert!(config.backends.systemd_networkd.emit_json);
    }

    #[test]
    fn test_parse_new_config_format() {
        let yaml = r#"
system:
  log_level: "debug"
  backend: "systemd-networkd"

monitoring:
  interfaces:
    - eth0
    - eth1
    - wlan0

routing:
  policy_rules:
    - eth1

backends:
  systemd_networkd:
    emit_json: true

  dhclient:
    use_dns: true
    use_domain: false
    use_hostname: false
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.system.log_level, "debug");
        assert_eq!(config.get_links(), vec!["eth0", "eth1", "wlan0"]);
        assert_eq!(config.get_routing_policy_links(), vec!["eth1"]);
        assert!(config.get_emit_json());
        assert!(config.get_use_dns());
        assert!(!config.get_use_domain());
    }

    #[test]
    fn test_should_monitor_link() {
        let yaml = r#"
monitoring:
  interfaces:
    - eth0
    - eth1
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
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

    #[test]
    fn test_routing_policy_rules() {
        let yaml = r#"
routing:
  policy_rules:
    - eth1
    - eth2
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.should_configure_routing_rules("eth1"));
        assert!(config.should_configure_routing_rules("eth2"));
        assert!(!config.should_configure_routing_rules("eth0"));
    }

    #[test]
    fn test_minimal_config() {
        let yaml = r#"
system:
  backend: "NetworkManager"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.system.backend, "NetworkManager");
        assert_eq!(config.system.log_level, "info");
        assert!(config.monitoring.interfaces.is_empty());
    }
}
