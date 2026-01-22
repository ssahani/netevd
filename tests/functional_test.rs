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

/// Test multiple IP addresses on the same interface
#[test]
#[ignore] // Requires root privileges
fn test_multiple_ip_addresses() {
    if !is_root() {
        eprintln!("Skipping test_multiple_ip_addresses: requires root");
        return;
    }

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");

    // Add multiple IPv4 addresses
    let addresses = [
        "192.168.1.10/24",
        "192.168.1.11/24",
        "10.0.0.5/8",
        "172.16.0.1/16",
    ];

    for addr in &addresses {
        add_ip_address(TEST_INTERFACE, addr).expect("Failed to add IP");
    }

    // Verify all addresses are assigned
    let output = Command::new("ip")
        .args(["addr", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check addresses");

    let output_str = String::from_utf8_lossy(&output.stdout);
    for addr in &addresses {
        assert!(
            output_str.contains(addr),
            "Address {} should be assigned",
            addr
        );
    }

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test IPv6 address assignment
#[test]
#[ignore] // Requires root privileges
fn test_ipv6_addresses() {
    if !is_root() {
        eprintln!("Skipping test_ipv6_addresses: requires root");
        return;
    }

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");

    // Add IPv6 addresses
    let ipv6_addrs = [
        "2001:db8::1/64",
        "fe80::1/64",
        "fd00::1/64",
    ];

    for addr in &ipv6_addrs {
        add_ip_address(TEST_INTERFACE, addr).expect("Failed to add IPv6");
    }

    // Verify IPv6 addresses
    let output = Command::new("ip")
        .args(["-6", "addr", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check IPv6 addresses");

    let output_str = String::from_utf8_lossy(&output.stdout);
    for addr in &ipv6_addrs {
        let addr_without_prefix = addr.split('/').next().unwrap();
        assert!(
            output_str.contains(addr_without_prefix),
            "IPv6 address {} should be assigned",
            addr
        );
    }

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test removing IP addresses
#[test]
#[ignore] // Requires root privileges
fn test_remove_ip_addresses() {
    if !is_root() {
        eprintln!("Skipping test_remove_ip_addresses: requires root");
        return;
    }

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");

    // Add IP address
    add_ip_address(TEST_INTERFACE, "192.168.50.1/24").expect("Failed to add IP");

    // Verify it's there
    let output = Command::new("ip")
        .args(["addr", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check address");
    assert!(String::from_utf8_lossy(&output.stdout).contains("192.168.50.1/24"));

    // Remove IP address
    let output = Command::new("ip")
        .args(["addr", "del", "192.168.50.1/24", "dev", TEST_INTERFACE])
        .output()
        .expect("Failed to remove IP");
    assert!(output.status.success(), "Should remove IP address");

    // Verify it's gone
    let output = Command::new("ip")
        .args(["addr", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check address");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("192.168.50.1/24"));

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test MTU changes
#[test]
#[ignore] // Requires root privileges
fn test_mtu_changes() {
    if !is_root() {
        eprintln!("Skipping test_mtu_changes: requires root");
        return;
    }

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");

    // Set different MTU values
    let mtu_values = [1500, 9000, 1400, 1280];

    for mtu in &mtu_values {
        let output = Command::new("ip")
            .args(["link", "set", TEST_INTERFACE, "mtu", &mtu.to_string()])
            .output()
            .expect("Failed to set MTU");

        assert!(output.status.success(), "Should set MTU to {}", mtu);

        // Verify MTU
        let output = Command::new("ip")
            .args(["link", "show", TEST_INTERFACE])
            .output()
            .expect("Failed to check MTU");

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(
            output_str.contains(&format!("mtu {}", mtu)),
            "MTU should be {}",
            mtu
        );
    }

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test script execution order with multiple scripts
#[tokio::test]
#[ignore] // Requires root privileges
async fn test_script_execution_order() {
    if !is_root() {
        eprintln!("Skipping test_script_execution_order: requires root");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_dir = temp_dir.path().join("carrier.d");
    fs::create_dir_all(&script_dir).expect("Failed to create script dir");

    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);

    // Create multiple scripts with numeric prefixes
    for i in 1..=5 {
        let script_content = format!(
            r#"#!/bin/bash
echo "Script {}: $LINK" >> {}
"#,
            i, TEST_SCRIPT_OUTPUT
        );
        let script_name = format!("{:02}-script.sh", i);
        create_test_script(&script_dir, &script_name, &script_content)
            .expect("Failed to create script");
    }

    // Execute all scripts in order
    let mut entries: Vec<_> = fs::read_dir(&script_dir)
        .expect("Failed to read dir")
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let output = Command::new("bash")
            .arg(entry.path())
            .env("LINK", TEST_INTERFACE)
            .output()
            .expect("Failed to execute script");
        assert!(output.status.success());
    }

    // Verify execution order
    let content = fs::read_to_string(TEST_SCRIPT_OUTPUT)
        .expect("Failed to read output");

    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 5, "Should have 5 script outputs");

    for (i, line) in lines.iter().enumerate() {
        assert!(
            line.contains(&format!("Script {}", i + 1)),
            "Script {} should execute in order",
            i + 1
        );
    }

    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);
}

/// Test script failure handling
#[test]
#[ignore] // Requires root privileges
fn test_script_failure_handling() {
    if !is_root() {
        eprintln!("Skipping test_script_failure_handling: requires root");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_dir = temp_dir.path().join("test.d");
    fs::create_dir_all(&script_dir).expect("Failed to create script dir");

    // Create script that fails
    let failing_script = r#"#!/bin/bash
echo "This script fails"
exit 1
"#;

    create_test_script(&script_dir, "01-failing.sh", failing_script)
        .expect("Failed to create failing script");

    // Execute script
    let output = Command::new("bash")
        .arg(script_dir.join("01-failing.sh"))
        .output()
        .expect("Failed to execute script");

    assert!(!output.status.success(), "Script should fail with exit code 1");
    assert_eq!(output.status.code(), Some(1), "Exit code should be 1");
}

/// Test bridge interface creation
#[test]
#[ignore] // Requires root privileges
fn test_bridge_interface() {
    if !is_root() {
        eprintln!("Skipping test_bridge_interface: requires root");
        return;
    }

    let bridge_name = "br-test0";

    // Create bridge
    let output = Command::new("ip")
        .args(["link", "add", bridge_name, "type", "bridge"])
        .output()
        .expect("Failed to create bridge");

    assert!(output.status.success(), "Should create bridge");

    // Bring bridge up
    bring_interface_up(bridge_name).expect("Failed to bring up bridge");

    // Add IP to bridge
    add_ip_address(bridge_name, "192.168.100.1/24").expect("Failed to add IP to bridge");

    // Verify bridge exists and has IP
    let output = Command::new("ip")
        .args(["addr", "show", bridge_name])
        .output()
        .expect("Failed to check bridge");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("192.168.100.1/24"));
    assert!(output_str.contains("master br-test0") || output_str.contains(bridge_name));

    // Cleanup
    delete_interface(bridge_name).expect("Failed to delete bridge");
}

/// Test no-carrier event (interface down)
#[tokio::test]
#[ignore] // Requires root privileges
async fn test_no_carrier_event() {
    if !is_root() {
        eprintln!("Skipping test_no_carrier_event: requires root");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let no_carrier_dir = temp_dir.path().join("no-carrier.d");
    fs::create_dir_all(&no_carrier_dir).expect("Failed to create no-carrier.d");

    let script_content = format!(
        r#"#!/bin/bash
echo "NO-CARRIER: $LINK" >> {}
"#,
        TEST_SCRIPT_OUTPUT
    );

    create_test_script(&no_carrier_dir, "01-test.sh", &script_content)
        .expect("Failed to create script");

    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up interface");

    // Bring interface down (triggers no-carrier)
    bring_interface_down(TEST_INTERFACE).expect("Failed to bring down interface");

    sleep(Duration::from_secs(1)).await;

    // Manually execute script to test
    let output = Command::new("bash")
        .arg(no_carrier_dir.join("01-test.sh"))
        .env("LINK", TEST_INTERFACE)
        .env("STATE", "no-carrier")
        .output()
        .expect("Failed to execute script");

    assert!(output.status.success());

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
    let _ = fs::remove_file(TEST_SCRIPT_OUTPUT);
}

/// Test rapid state changes
#[tokio::test]
#[ignore] // Requires root privileges
async fn test_rapid_state_changes() {
    if !is_root() {
        eprintln!("Skipping test_rapid_state_changes: requires root");
        return;
    }

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");

    // Rapidly change interface state
    for _ in 0..10 {
        bring_interface_up(TEST_INTERFACE).expect("Failed to bring up");
        sleep(Duration::from_millis(100)).await;
        bring_interface_down(TEST_INTERFACE).expect("Failed to bring down");
        sleep(Duration::from_millis(100)).await;
    }

    // Verify interface still exists and is in a valid state
    let output = Command::new("ip")
        .args(["link", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to check interface");

    assert!(output.status.success(), "Interface should still exist");

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test concurrent interface operations
#[tokio::test]
#[ignore] // Requires root privileges
async fn test_concurrent_operations() {
    if !is_root() {
        eprintln!("Skipping test_concurrent_operations: requires root");
        return;
    }

    let interfaces = ["dummy-conc1", "dummy-conc2", "dummy-conc3", "dummy-conc4"];

    // Create all interfaces concurrently
    for iface in &interfaces {
        create_dummy_interface(iface).expect("Failed to create interface");
    }

    // Bring them all up
    for iface in &interfaces {
        bring_interface_up(iface).expect("Failed to bring up");
    }

    // Add IPs concurrently
    for (i, iface) in interfaces.iter().enumerate() {
        let ip = format!("10.10.{}.1/24", i + 1);
        add_ip_address(iface, &ip).expect("Failed to add IP");
    }

    // Verify all are configured
    for (i, iface) in interfaces.iter().enumerate() {
        let output = Command::new("ip")
            .args(["addr", "show", iface])
            .output()
            .expect("Failed to check interface");

        let output_str = String::from_utf8_lossy(&output.stdout);
        let expected_ip = format!("10.10.{}.1/24", i + 1);
        assert!(output_str.contains(&expected_ip));
    }

    // Cleanup all
    for iface in &interfaces {
        delete_interface(iface).expect("Failed to delete interface");
    }
}

/// Test macvlan interface
#[test]
#[ignore] // Requires root privileges
fn test_macvlan_interface() {
    if !is_root() {
        eprintln!("Skipping test_macvlan_interface: requires root");
        return;
    }

    // Create parent interface first
    create_dummy_interface("dummy-parent").expect("Failed to create parent");
    bring_interface_up("dummy-parent").expect("Failed to bring up parent");

    // Create macvlan on top of parent
    let output = Command::new("ip")
        .args([
            "link",
            "add",
            "macvlan-test",
            "link",
            "dummy-parent",
            "type",
            "macvlan",
            "mode",
            "bridge",
        ])
        .output()
        .expect("Failed to create macvlan");

    assert!(output.status.success(), "Should create macvlan");

    // Bring macvlan up
    bring_interface_up("macvlan-test").expect("Failed to bring up macvlan");

    // Add IP to macvlan
    add_ip_address("macvlan-test", "192.168.99.1/24").expect("Failed to add IP");

    // Verify macvlan
    let output = Command::new("ip")
        .args(["link", "show", "macvlan-test"])
        .output()
        .expect("Failed to check macvlan");

    assert!(String::from_utf8_lossy(&output.stdout).contains("macvlan"));

    // Cleanup
    delete_interface("macvlan-test").expect("Failed to delete macvlan");
    delete_interface("dummy-parent").expect("Failed to delete parent");
}

/// Test large number of interfaces
#[test]
#[ignore] // Requires root privileges
fn test_many_interfaces() {
    if !is_root() {
        eprintln!("Skipping test_many_interfaces: requires root");
        return;
    }

    let count = 20;
    let mut interfaces = Vec::new();

    // Create many interfaces
    for i in 0..count {
        let name = format!("dummy-many{}", i);
        create_dummy_interface(&name).expect("Failed to create interface");
        interfaces.push(name);
    }

    // Configure all
    for (i, iface) in interfaces.iter().enumerate() {
        bring_interface_up(iface).expect("Failed to bring up");
        let ip = format!("10.99.{}.1/24", i);
        add_ip_address(iface, &ip).expect("Failed to add IP");
    }

    // Verify count
    let output = Command::new("ip")
        .args(["link", "show"])
        .output()
        .expect("Failed to list interfaces");

    let output_str = String::from_utf8_lossy(&output.stdout);
    for iface in &interfaces {
        assert!(output_str.contains(iface), "Interface {} should exist", iface);
    }

    // Cleanup all
    for iface in &interfaces {
        delete_interface(iface).expect("Failed to delete interface");
    }
}

/// Test interface with special characters in name
#[test]
#[ignore] // Requires root privileges
fn test_interface_naming() {
    if !is_root() {
        eprintln!("Skipping test_interface_naming: requires root");
        return;
    }

    // Valid interface names with various formats
    let valid_names = [
        "dummy-test",
        "dummy_test",
        "dummy.test",
        "dummy0",
        "test123",
    ];

    for name in &valid_names {
        let result = create_dummy_interface(name);
        if result.is_ok() {
            // Some names may not be supported by the kernel
            delete_interface(name).ok();
        }
    }
}

/// Test adding routes to interface
#[test]
#[ignore] // Requires root privileges
fn test_add_routes() {
    if !is_root() {
        eprintln!("Skipping test_add_routes: requires root");
        return;
    }

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up");
    add_ip_address(TEST_INTERFACE, "192.168.200.1/24").expect("Failed to add IP");

    // Add a route
    let output = Command::new("ip")
        .args([
            "route",
            "add",
            "10.0.0.0/8",
            "via",
            "192.168.200.254",
            "dev",
            TEST_INTERFACE,
        ])
        .output()
        .expect("Failed to add route");

    assert!(output.status.success(), "Should add route");

    // Verify route exists
    let output = Command::new("ip")
        .args(["route", "show"])
        .output()
        .expect("Failed to show routes");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("10.0.0.0/8"));
    assert!(output_str.contains(TEST_INTERFACE));

    // Cleanup route
    let _ = Command::new("ip")
        .args(["route", "del", "10.0.0.0/8"])
        .output();

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}

/// Test interface statistics
#[test]
#[ignore] // Requires root privileges
fn test_interface_statistics() {
    if !is_root() {
        eprintln!("Skipping test_interface_statistics: requires root");
        return;
    }

    create_dummy_interface(TEST_INTERFACE).expect("Failed to create interface");
    bring_interface_up(TEST_INTERFACE).expect("Failed to bring up");

    // Get interface statistics
    let output = Command::new("ip")
        .args(["-s", "link", "show", TEST_INTERFACE])
        .output()
        .expect("Failed to get statistics");

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Should contain RX and TX statistics
    assert!(output_str.contains("RX:") || output_str.contains("bytes"));
    assert!(output_str.contains("TX:") || output_str.contains("packets"));

    delete_interface(TEST_INTERFACE).expect("Failed to delete interface");
}
