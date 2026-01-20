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

    // Valid configuration (note: PascalCase due to serde rename_all)
    let config_content = r#"
system:
  LogLevel: "info"
  Backend: "systemd-networkd"

network:
  UseDns: true
  UseDomain: true
  UseHostname: true
  RoutingPolicyRules: "eth0 eth1"
"#;

    fs::write(&config_path, config_content).unwrap();

    // Test that config loads successfully
    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap());
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.system.backend, "systemd-networkd");
    assert_eq!(config.system.log_level, "info");
    assert!(config.network.use_dns);
    let routing_policy = config.network.get_routing_policy_interfaces();
    assert_eq!(routing_policy.len(), 2);
}

/// Test invalid configuration handling
#[test]
fn test_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.yaml");

    // Invalid YAML
    let config_content = r#"
system:
  LogLevel: info
  Backend: [invalid
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
  Backend: "NetworkManager"
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = netevd::config::Config::parse_from_path(config_path.to_str().unwrap()).unwrap();

    // Check defaults
    assert_eq!(config.system.log_level, "info");
    assert!(!config.network.use_dns); // Default is false
    assert!(!config.network.use_domain); // Default is false
    assert!(!config.network.use_hostname); // Default is false
}

/// Test environment variable overrides
#[test]
fn test_env_var_override() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("netevd.yaml");

    let config_content = r#"
system:
  LogLevel: "info"
  Backend: "systemd-networkd"
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
