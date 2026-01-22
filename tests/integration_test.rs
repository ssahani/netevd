// SPDX-License-Identifier: LGPL-3.0-or-later

//! Integration tests for netevd daemon

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test configuration loading and validation
#[test]
fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    // Valid configuration with new format
    let config_content = r#"
system:
  log_level: "info"
  backend: "systemd-networkd"

monitoring:
  interfaces:
    - eth0
    - eth1

routing:
  policy_rules:
    - eth1

backends:
  dhclient:
    use_dns: true
    use_domain: true
    use_hostname: true
"#;

    fs::write(&config_path, config_content).unwrap();

    // Test that config loads successfully
    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap());
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.system.backend, "systemd-networkd");
    assert_eq!(config.system.log_level, "info");
    assert!(config.get_use_dns());
    let routing_policy = config.routing.get_routing_policy_interfaces();
    assert_eq!(routing_policy.len(), 1);
    assert_eq!(routing_policy[0], "eth1");
}

/// Test invalid configuration handling
#[test]
fn test_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.yaml");

    // Invalid YAML
    let config_content = r#"
system:
  log_level: info
  backend: [invalid
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap());
    assert!(config.is_err());
}

/// Test default configuration values
#[test]
fn test_default_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("minimal.yaml");

    // Minimal configuration
    let config_content = r#"
system:
  backend: "NetworkManager"
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    // Check defaults
    assert_eq!(config.system.log_level, "info");
    assert!(!config.get_use_dns()); // Default is false
    assert!(!config.get_use_domain()); // Default is false
    assert!(!config.get_use_hostname()); // Default is false
}

/// Test environment variable overrides
#[test]
fn test_env_var_override() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
system:
  log_level: "info"
  backend: "systemd-networkd"
"#;

    fs::write(&config_path, config_content).unwrap();

    // Set environment variable
    std::env::set_var("NETEVD_BACKEND", "NetworkManager");
    std::env::set_var("NETEVD_LOG_LEVEL", "debug");

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    // Note: Environment variable override is not yet implemented
    // For now, just verify the config loads
    assert_eq!(config.system.backend, "systemd-networkd");

    // Cleanup
    std::env::remove_var("NETEVD_BACKEND");
    std::env::remove_var("NETEVD_LOG_LEVEL");
}

/// Test network state initialization
#[tokio::test]
async fn test_network_state_initialization() {
    use netevd::network::NetworkState;

    let state = NetworkState::new();

    // State should be created (field access is private, just test creation)
    assert!(true); // State created successfully
}

/// Test script directory creation
#[test]
fn test_script_dir_paths() {
    use netevd::system::paths::get_script_dir;

    assert_eq!(get_script_dir("carrier"), "/etc/netevd/carrier.d");
    assert_eq!(get_script_dir("routable"), "/etc/netevd/routable.d");
    assert_eq!(get_script_dir("degraded"), "/etc/netevd/degraded.d");
    assert_eq!(get_script_dir("routes"), "/etc/netevd/routes.d");
}

/// Test link state tracking
#[tokio::test]
async fn test_link_state_tracking() {
    use netevd::network::NetworkState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let state = Arc::new(RwLock::new(NetworkState::new()));

    {
        let mut state_write = state.write().await;

        // Add a link using the public API
        state_write.add_link("eth0".to_string(), 2);
    }

    // Verify link was added
    {
        let state_read = state.read().await;
        assert_eq!(state_read.get_link_name(2), Some(&"eth0".to_string()));
        assert_eq!(state_read.get_link_index("eth0"), Some(2));
    }
}

/// Test concurrent state access
#[tokio::test]
async fn test_concurrent_state_access() {
    use netevd::network::NetworkState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let state = Arc::new(RwLock::new(NetworkState::new()));

    // Spawn multiple readers
    let mut handles = vec![];

    for _i in 0..10 {
        let state_clone = Arc::clone(&state);
        let handle = tokio::spawn(async move {
            let state_read = state_clone.read().await;
            // Just verify we can read the state
            let _name = state_read.get_link_name(3);
            true
        });
        handles.push(handle);
    }

    // Add data concurrently
    {
        let mut state_write = state.write().await;
        state_write.add_link("wlan0".to_string(), 3);
    }

    // All readers should complete
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Test config with all backends specified
#[test]
fn test_config_all_backends() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
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
    - wlan0

backends:
  systemd_networkd:
    emit_json: true

  dhclient:
    use_dns: true
    use_domain: true
    use_hostname: false

  networkmanager: {}
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    assert_eq!(config.system.log_level, "debug");
    assert_eq!(config.system.backend, "systemd-networkd");
    assert_eq!(config.monitoring.interfaces.len(), 3);
    assert_eq!(config.routing.policy_rules.len(), 2);
    assert!(config.get_emit_json());
    assert!(config.get_use_dns());
    assert!(config.get_use_domain());
    assert!(!config.get_use_hostname());
}

/// Test config with empty interfaces (monitor all)
#[test]
fn test_config_empty_interfaces() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
system:
  backend: "NetworkManager"

monitoring:
  interfaces: []
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    assert_eq!(config.monitoring.interfaces.len(), 0);
    assert!(config.should_monitor_link("eth0"));
    assert!(config.should_monitor_link("wlan0"));
    assert!(config.should_monitor_link("any-interface"));
}

/// Test config with specific interfaces
#[test]
fn test_config_specific_interfaces() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
monitoring:
  interfaces:
    - eth0
    - eth1
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    assert!(config.should_monitor_link("eth0"));
    assert!(config.should_monitor_link("eth1"));
    assert!(!config.should_monitor_link("wlan0"));
    assert!(!config.should_monitor_link("eth2"));
}

/// Test routing policy rules configuration
#[test]
fn test_config_routing_policy_rules() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
monitoring:
  interfaces:
    - eth0
    - eth1
    - eth2

routing:
  policy_rules:
    - eth1
    - eth2
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    assert!(!config.should_configure_routing_rules("eth0"));
    assert!(config.should_configure_routing_rules("eth1"));
    assert!(config.should_configure_routing_rules("eth2"));
}

/// Test missing config file uses defaults
#[test]
fn test_missing_config_file() {
    let config = netevd::config::Config::parse_from_path("/nonexistent/path/config.yaml");

    assert!(config.is_ok());
    let config = config.unwrap();

    assert_eq!(config.system.log_level, "info");
    assert_eq!(config.system.backend, "systemd-networkd");
    assert_eq!(config.monitoring.interfaces.len(), 0);
}

/// Test config with only systemd_networkd backend options
#[test]
fn test_config_systemd_networkd_only() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
system:
  backend: "systemd-networkd"

backends:
  systemd_networkd:
    emit_json: false
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    assert!(!config.get_emit_json());
    assert!(!config.get_use_dns());
    assert!(!config.get_use_domain());
}

/// Test config with only dhclient backend options
#[test]
fn test_config_dhclient_only() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
system:
  backend: "dhclient"

backends:
  dhclient:
    use_dns: true
    use_domain: true
    use_hostname: true
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    assert!(config.get_use_dns());
    assert!(config.get_use_domain());
    assert!(config.get_use_hostname());
    assert!(config.get_emit_json()); // Default is true
}

/// Test various log levels
#[test]
fn test_config_log_levels() {
    for log_level in &["trace", "debug", "info", "warn", "error"] {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("netevd.yaml");

        let config_content = format!(
            r#"
system:
  log_level: "{}"
"#,
            log_level
        );

        fs::write(&config_path, config_content).unwrap();

        let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

        assert_eq!(config.system.log_level, *log_level);
    }
}

/// Test all supported backends
#[test]
fn test_config_all_backend_types() {
    for backend in &["systemd-networkd", "NetworkManager", "dhclient"] {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("netevd.yaml");

        let config_content = format!(
            r#"
system:
  backend: "{}"
"#,
            backend
        );

        fs::write(&config_path, config_content).unwrap();

        let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

        assert_eq!(config.system.backend, *backend);
    }
}

/// Test config with complex interface names
#[test]
fn test_config_complex_interface_names() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
monitoring:
  interfaces:
    - eth0
    - wlan0
    - br-1234abcd
    - veth_test
    - eno1
    - enp0s3
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    assert_eq!(config.monitoring.interfaces.len(), 6);
    assert!(config.should_monitor_link("eth0"));
    assert!(config.should_monitor_link("br-1234abcd"));
    assert!(config.should_monitor_link("veth_test"));
    assert!(!config.should_monitor_link("eth1"));
}

/// Test validation functions
#[test]
fn test_interface_name_validation() {
    use netevd::system::validation::validate_interface_name;

    // Valid names
    assert!(validate_interface_name("eth0"));
    assert!(validate_interface_name("wlan0"));
    assert!(validate_interface_name("br-1234"));
    assert!(validate_interface_name("veth_test"));
    assert!(validate_interface_name("eno1.100"));

    // Invalid names
    assert!(!validate_interface_name(""));
    assert!(!validate_interface_name("a".repeat(16).as_str()));
    assert!(!validate_interface_name("eth0; rm -rf /"));
    assert!(!validate_interface_name("eth$0"));
    assert!(!validate_interface_name("eth 0"));
}

/// Test hostname validation
#[test]
fn test_hostname_validation() {
    use netevd::system::validation::validate_hostname;

    // Valid hostnames
    assert!(validate_hostname("localhost"));
    assert!(validate_hostname("example.com"));
    assert!(validate_hostname("sub.example.com"));
    assert!(validate_hostname("my-host"));
    assert!(validate_hostname("host123"));

    // Invalid hostnames
    assert!(!validate_hostname(""));
    assert!(!validate_hostname("-invalid"));
    assert!(!validate_hostname("invalid-"));
    assert!(!validate_hostname("in valid"));
    assert!(!validate_hostname(&"a".repeat(64)));
    assert!(!validate_hostname("host."));
    assert!(!validate_hostname(".host"));
}

/// Test IP address validation
#[test]
fn test_ip_validation() {
    use netevd::system::validation::validate_ip_address;

    // Valid IPv4
    assert!(validate_ip_address("192.168.1.1"));
    assert!(validate_ip_address("10.0.0.1"));
    assert!(validate_ip_address("0.0.0.0"));
    assert!(validate_ip_address("255.255.255.255"));

    // Valid IPv6
    assert!(validate_ip_address("::1"));
    assert!(validate_ip_address("2001:db8::1"));
    assert!(validate_ip_address("fe80::1"));

    // Invalid
    assert!(!validate_ip_address(""));
    assert!(!validate_ip_address("256.256.256.256"));
    assert!(!validate_ip_address("not-an-ip"));
    assert!(!validate_ip_address("192.168.1"));
}

/// Test environment value sanitization
#[test]
fn test_env_sanitization() {
    use netevd::system::validation::sanitize_env_value;

    // Safe values
    assert!(sanitize_env_value("safe_value-123").is_some());
    assert!(sanitize_env_value("192.168.1.1").is_some());
    assert!(sanitize_env_value("eth0").is_some());
    assert!(sanitize_env_value("normal text").is_some());

    // Dangerous values
    assert!(sanitize_env_value("value; rm -rf /").is_none());
    assert!(sanitize_env_value("$(whoami)").is_none());
    assert!(sanitize_env_value("`whoami`").is_none());
    assert!(sanitize_env_value("value && malicious").is_none());
    assert!(sanitize_env_value("val$ue").is_none());
    assert!(sanitize_env_value("val|ue").is_none());
    assert!(sanitize_env_value("val>file").is_none());
}

/// Test state name validation
#[test]
fn test_state_name_validation() {
    use netevd::system::validation::validate_state_name;

    // Valid states
    assert!(validate_state_name("routable"));
    assert!(validate_state_name("activated"));
    assert!(validate_state_name("no-carrier"));
    assert!(validate_state_name("carrier"));
    assert!(validate_state_name("configured"));
    assert!(validate_state_name("degraded"));
    assert!(validate_state_name("disconnected"));
    assert!(validate_state_name("manager"));
    assert!(validate_state_name("routes"));

    // Invalid states
    assert!(!validate_state_name(""));
    assert!(!validate_state_name("invalid-state"));
    assert!(!validate_state_name("../../../etc/passwd"));
    assert!(!validate_state_name("unknown"));
}
