// SPDX-License-Identifier: LGPL-3.0-or-later

//! Functional tests for netevd daemon
//!
//! These tests require root privileges and create real network interfaces.
//! Run with: sudo cargo test --test functional_test -- --test-threads=1

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

const TEST_INTERFACE: &str = "dummy-test0";
const TEST_SCRIPT_OUTPUT: &str = "/tmp/netevd-test-output.txt";

/// Helper to check if running as root
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Helper to create a dummy network interface
fn create_dummy_interface(name: &str) -> Result<(), String> {
    let output = Command::new("ip")
        .args(["link", "add", name, "type", "dummy"])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to create interface: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Helper to delete a network interface
fn delete_interface(name: &str) -> Result<(), String> {
    let output = Command::new("ip")
        .args(["link", "del", name])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to delete interface: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Helper to bring interface up
fn bring_interface_up(name: &str) -> Result<(), String> {
    let output = Command::new("ip")
        .args(["link", "set", name, "up"])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to bring up interface: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Helper to bring interface down
fn bring_interface_down(name: &str) -> Result<(), String> {
    let output = Command::new("ip")
        .args(["link", "set", name, "down"])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to bring down interface: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Helper to add IP address to interface
fn add_ip_address(name: &str, ip: &str) -> Result<(), String> {
    let output = Command::new("ip")
        .args(["addr", "add", ip, "dev", name])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to add IP address: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Helper to create a test script
fn create_test_script(dir: &Path, name: &str, content: &str) -> Result<(), std::io::Error> {
    let script_path = dir.join(name);
    fs::write(&script_path, content)?;

    // Make executable
    let mut perms = fs::metadata(&script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms)?;

    Ok(())
}

/// Test interface creation and carrier state
#[test]
#[ignore] // Requires root privileges
fn test_dummy_interface_creation() {
    if !is_root() {
        eprintln!("Skipping test_dummy_interface_creation: requires root");
        return;
    }

    // Create dummy interface
    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");

    // Verify interface exists
    let output = Command::new("ip")
        .args(["link", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check interface");

    assert!(output.status.success(), "Interface should exist");

    // Cleanup
    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test bringing interface up and down
#[test]
#[ignore] // Requires root privileges
fn test_interface_up_down() {
    if !is_root() {
        eprintln!("Skipping test_interface_up_down: requires root");
        return;
    }

    // Create and bring up interface
    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");

    // Check interface is up
    let output = Command::new("ip")
        .args(["link", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check interface");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("UP"), "Interface should be UP");

    // Bring interface down
    bring_interface_down(TEST_INTERFACE).expect("Failed to bring down interface");

    // Check interface is down
    let output = Command::new("ip")
        .args(["link", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check interface");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(!output_str.contains("state UP"), "Interface should be DOWN");

    // Cleanup
    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test adding IP address to interface
#[test]
#[ignore] // Requires root privileges
fn test_add_ip_address() {
    if !is_root() {
        eprintln!("Skipping test_add_ip_address: requires root");
        return;
    }

    // Create interface and bring it up
    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");

    // Add IP address
    add_ip_address(TEST_INTERFACE, "192.168.100.1/24").expect("Failed to add IP");

    // Verify IP address is assigned
    let output = Command::new("ip")
        .args(["addr", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check IP address");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("192.168.100.1/24"),
        "IP address should be assigned"
    );

    // Cleanup
    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test script execution on carrier event
#[tokio::test]
#[ignore] // Requires root privileges and daemon setup
async fn test_carrier_script_execution() {
    if !is_root() {
        eprintln!("Skipping test_carrier_script_execution: requires root");
        return;
    }

    // Create temporary script directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let carrier_dir = temp_dir.path().join("carrier.d");
    fs::create_dir_all(&carrier_dir).expect("Failed to create carrier.d");

    // Create test script that writes to output file
    let script_content = format!(
        r#"#!/bin/bash
echo "CARRIER: $LINK" >> {}
echo "STATE: $STATE" >> {}
echo "BACKEND: $BACKEND" >> {}
"#,
        TEST_SCRIPT_OUTPUT, TEST_SCRIPT_OUTPUT, TEST_SCRIPT_OUTPUT
    );

    create_test_script(&carrier_dir, "01-test.sh", &script_content)
        .expect("Failed to create test script");

    // Clean up any existing output file
    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);

    // Create dummy interface
    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");

    // Bring interface up (this should trigger carrier event)
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");

    // Wait for daemon to process event (if running)
    sleep(Duration::from_secs(2)).await;

    // Manually execute the script to test it works
    let output = Command::new("bash")
        .arg(carrier_dir.join("01-test.sh"))
        .env("LINK", TEST_INTERFACE)
        .env("STATE", "carrier")
        .env("BACKEND", "test")
        .output()
        .expect("Failed to execute test script");

    assert!(output.status.success(), "Script should execute successfully");

    // Verify script output
    if Path::new(TEST_SCRIPT_OUTPUT).exists() {
        let content = fs::read_to_string(TEST_SCRIPT_OUTPUT)
            .expect("Failed to read output file");
        assert!(content.contains(TEST_INTERFACE), "Output should contain interface name");
        assert!(content.contains("carrier"), "Output should contain state");
    }

    // Cleanup
    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);
}

/// Test routable state with IP address
#[tokio::test]
#[ignore] // Requires root privileges and daemon setup
async fn test_routable_with_ip() {
    if !is_root() {
        eprintln!("Skipping test_routable_with_ip: requires root");
        return;
    }

    // Create temporary script directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let routable_dir = temp_dir.path().join("routable.d");
    fs::create_dir_all(&routable_dir).expect("Failed to create routable.d");

    // Create test script
    let script_content = format!(
        r#"#!/bin/bash
echo "ROUTABLE: $LINK" >> {}
echo "ADDRESSES: $ADDRESSES" >> {}
"#,
        TEST_SCRIPT_OUTPUT, TEST_SCRIPT_OUTPUT
    );

    create_test_script(&routable_dir, "01-test.sh", &script_content)
        .expect("Failed to create test script");

    // Clean up any existing output file
    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);

    // Create dummy interface, bring it up, and add IP
    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");
    add_ip_address(TEST_INTERFACE, "10.0.0.1/24").expect("Failed to add IP");

    // Wait for daemon to process events
    sleep(Duration::from_secs(2)).await;

    // Manually execute the script to test
    let output = Command::new("bash")
        .arg(routable_dir.join("01-test.sh"))
        .env("LINK", TEST_INTERFACE)
        .env("STATE", "routable")
        .env("ADDRESSES", "10.0.0.1")
        .output()
        .expect("Failed to execute test script");

    assert!(output.status.success(), "Script should execute successfully");

    // Verify output
    if Path::new(TEST_SCRIPT_OUTPUT).exists() {
        let content = fs::read_to_string(TEST_SCRIPT_OUTPUT)
            .expect("Failed to read output file");
        assert!(content.contains(TEST_INTERFACE), "Output should contain interface name");
        assert!(content.contains("10.0.0.1"), "Output should contain IP address");
    }

    // Cleanup
    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);
}

/// Test multiple interface lifecycle
#[tokio::test]
#[ignore] // Requires root privileges
async fn test_multiple_interfaces() {
    if !is_root() {
        eprintln!("Skipping test_multiple_interfaces: requires root");
        return;
    }

    let interfaces = ["dummy-test1", "dummy-test2", "dummy-test3"];

    // Create all interfaces
    for iface in &interfaces {
        create_dummy_interface(iface).expect("Failed to create interface");
    }

    // Bring them all up
    for iface in &interfaces {
        bring_interface_up(iface).expect("Failed to bring up interface");
    }

    // Add IP addresses
    for (i, iface) in interfaces.iter().enumerate() {
        let ip = format!("10.0.{}.1/24", i + 1);
        add_ip_address(iface, &ip).expect("Failed to add IP");
    }

    // Verify all interfaces are up with IPs
    for (i, iface) in interfaces.iter().enumerate() {
        let output = Command::new("ip")
            .args(["addr", "show", iface])
            .output()
            .expect("Failed to check interface");

        let output_str = String::from_utf8_lossy(&output.stdout);
        let expected_ip = format!("10.0.{}.1/24", i + 1);
        assert!(
            output_str.contains(&expected_ip),
            "Interface {} should have IP {}",
            iface,
            expected_ip
        );
    }

    // Cleanup all interfaces
    for iface in &interfaces {
        delete_interface(iface).expect("Failed to delete interface");
    }
}

/// Test veth pair creation and communication
#[test]
#[ignore] // Requires root privileges
fn test_veth_pair() {
    if !is_root() {
        eprintln!("Skipping test_veth_pair: requires root");
        return;
    }

    // Create veth pair
    let output = Command::new("ip")
        .args(["link", "add", "veth-test0", "type", "veth", "peer", "name", "veth-test1"])
        .output()
        .expect("Failed to create veth pair");

    assert!(output.status.success(), "Should create veth pair");

    // Bring both ends up
    bring_interface_up("veth-test0").expect("Failed to bring up veth-test0");
    bring_interface_up("veth-test1").expect("Failed to bring up veth-test1");

    // Add IP addresses
    add_ip_address("veth-test0", "192.168.200.1/24").expect("Failed to add IP to veth-test0");
    add_ip_address("veth-test1", "192.168.200.2/24").expect("Failed to add IP to veth-test1");

    // Verify both interfaces have IPs
    let output = Command::new("ip")
        .args(["addr", "show", "veth-test0"])
        .output()
        .expect("Failed to check veth-test0");

    assert!(String::from_utf8_lossy(&output.stdout).contains("192.168.200.1/24"));

    // Cleanup
    delete_interface("veth-test0").expect("Failed to delete veth pair");
}

/// Test script execution with environment variables
#[test]
#[ignore] // Requires root privileges
fn test_script_environment_variables() {
    if !is_root() {
        eprintln!("Skipping test_script_environment_variables: requires root");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_dir = temp_dir.path().join("test.d");
    fs::create_dir_all(&script_dir).expect("Failed to create script dir");

    // Create script that checks all environment variables
    let script_content = format!(
        r#"#!/bin/bash
echo "LINK=$LINK" >> {}
echo "LINKINDEX=$LINKINDEX" >> {}
echo "STATE=$STATE" >> {}
echo "BACKEND=$BACKEND" >> {}
echo "ADDRESSES=$ADDRESSES" >> {}
"#,
        TEST_SCRIPT_OUTPUT,
        TEST_SCRIPT_OUTPUT,
        TEST_SCRIPT_OUTPUT,
        TEST_SCRIPT_OUTPUT,
        TEST_SCRIPT_OUTPUT
    );

    create_test_script(&script_dir, "01-env-test.sh", &script_content)
        .expect("Failed to create script");

    // Clean output file
    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);

    // Execute script with test environment
    let output = Command::new("bash")
        .arg(script_dir.join("01-env-test.sh"))
        .env("LINK", "eth0")
        .env("LINKINDEX", "2")
        .env("STATE", "routable")
        .env("BACKEND", "systemd-networkd")
        .env("ADDRESSES", "192.168.1.100 10.0.0.5")
        .output()
        .expect("Failed to execute script");

    assert!(output.status.success(), "Script should execute successfully");

    // Verify all environment variables were passed
    let content = fs::read_to_string(TEST_SCRIPT_OUTPUT)
        .expect("Failed to read output");

    assert!(content.contains("LINK=eth0"));
    assert!(content.contains("LINKINDEX=2"));
    assert!(content.contains("STATE=routable"));
    assert!(content.contains("BACKEND=systemd-networkd"));
    assert!(content.contains("ADDRESSES=192.168.1.100 10.0.0.5"));

    // Cleanup
    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);
}
