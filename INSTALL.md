# Installation Guide for netevd

This document provides comprehensive installation instructions for netevd (Network Event Daemon).

## Table of Contents

- [Prerequisites](#prerequisites)
- [User Setup](#user-setup)
- [Building from Source](#building-from-source)
- [Installation](#installation)
- [Configuration](#configuration)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- Linux kernel 3.2 or later (for netlink support)
- systemd (optional, for systemd-networkd/resolved integration)
- Rust 1.70 or later
- Cargo package manager

### For systemd-networkd mode:
- `systemd-networkd` running
- `systemd-resolved` (optional, for DNS management)

### For NetworkManager mode:
- `NetworkManager` running
- DBus system bus

### For dhclient mode:
- `dhclient` installed and running
- Write access to `/var/lib/dhclient/`

## User Setup

netevd runs as an unprivileged user with limited capabilities for security. You must create the `netevd` user before running the daemon.

### Create the netevd User

```bash
# Create system user without home directory or login shell
sudo useradd --system \
             --no-create-home \
             --shell /usr/sbin/nologin \
             --comment "Network Event Daemon" \
             netevd
```

### Verify User Creation

```bash
# Check user was created
id netevd

# Should output something like:
# uid=996(netevd) gid=994(netevd) groups=994(netevd)
```

### User Details

- **Username:** `netevd`
- **Type:** System user (UID < 1000)
- **Home directory:** None (system user)
- **Login shell:** `/usr/sbin/nologin` (no interactive login)
- **Groups:** `netevd` (primary group)
- **Capabilities:** CAP_NET_ADMIN (granted by systemd service file)

### Manual User Configuration (Alternative)

If the above command doesn't work on your system:

```bash
# Create group first
sudo groupadd --system netevd

# Create user with specific UID/GID if needed
sudo useradd --system \
             --gid netevd \
             --no-create-home \
             --home-dir /nonexistent \
             --shell /usr/sbin/nologin \
             --comment "Network Event Daemon" \
             netevd
```

### Why a Dedicated User?

netevd runs as a dedicated user for security isolation:

1. **Principle of Least Privilege:** Runs with minimal permissions
2. **Capability Isolation:** Only CAP_NET_ADMIN is granted, not full root
3. **Audit Trail:** Actions can be attributed to the `netevd` user
4. **Process Isolation:** Cannot access other users' files or processes

## Building from Source

### Clone Repository

```bash
git clone https://github.com/ssahani/netevd.git
cd netevd
```

### Build Release Binary

```bash
# Build optimized release binary
cargo build --release

# Binary will be at: target/release/netevd
```

### Run Tests

```bash
# Run test suite
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_validate_interface_name
```

### Check for Issues

```bash
# Run clippy for linting
cargo clippy

# Check formatting
cargo fmt -- --check

# Format code
cargo fmt
```

## Installation

### Install Binary

```bash
# Install to /usr/bin (requires root)
sudo install -Dm755 target/release/netevd /usr/bin/netevd

# Verify installation
which netevd
netevd --version  # Will fail without config, but shows it's installed
```

### Install Configuration

```bash
# Create configuration directory
sudo mkdir -p /etc/netevd

# Install default configuration
sudo install -Dm644 distribution/netevd.yaml /etc/netevd/netevd.yaml

# Create script directories
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}
```

### Install systemd Service

```bash
# Install service file
sudo install -Dm644 distribution/netevd.service /lib/systemd/system/netevd.service

# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable netevd

# Start service
sudo systemctl start netevd

# Check status
sudo systemctl status netevd
```

### Set Permissions

```bash
# Configuration directory should be readable by netevd user
sudo chown -R root:root /etc/netevd
sudo chmod 755 /etc/netevd
sudo chmod 644 /etc/netevd/netevd.yaml

# Script directories should be readable and executable
sudo chmod 755 /etc/netevd/*.d
```

## Configuration

### Basic Configuration

Edit `/etc/netevd/netevd.yaml`:

```yaml
system:
  # Logging level: trace, debug, info, warn, error
  log_level: "info"

  # Network event generator: systemd-networkd, NetworkManager, or dhclient
  generator: "systemd-networkd"

network:
  # Space-separated list of interfaces to monitor (empty = all)
  links: "eth0 eth1"

  # Interfaces that need custom routing tables
  routing_policy_rules: "eth1"

  # Emit JSON data for systemd-networkd events
  emit_json: true

  # Send DNS to systemd-resolved (dhclient only)
  use_dns: false

  # Send domain to systemd-resolved (dhclient only)
  use_domain: false

  # Send hostname to systemd-hostnamed (dhclient only)
  use_hostname: false
```

### Create Example Scripts

Example routable script (`/etc/netevd/routable.d/01-notify.sh`):

```bash
#!/bin/bash
# This script runs when an interface becomes routable

echo "Interface $LINK is now routable with IP: $ADDRESSES"
logger -t netevd "Interface $LINK routable: $ADDRESSES"

# Add your custom logic here
```

Make it executable:

```bash
sudo chmod +x /etc/netevd/routable.d/01-notify.sh
```

### Restart After Configuration Changes

```bash
sudo systemctl restart netevd
```

## Testing

### Test User Permissions

```bash
# Switch to netevd user (will fail if properly configured - that's good!)
sudo su - netevd
# Should output: "This account is currently not available."

# Check user can access config
sudo -u netevd cat /etc/netevd/netevd.yaml
```

### Test Binary Execution

```bash
# Run in foreground with debug logging
sudo RUST_LOG=debug /usr/bin/netevd

# Should output:
# INFO Starting netevd - Network Event Daemon
# INFO Configuration loaded: ...
# INFO Dropping privileges to user 'netevd'
# ...
```

### Test Network Events

#### For systemd-networkd:

```bash
# Trigger network event
sudo networkctl reload

# Watch logs
sudo journalctl -u netevd -f
```

#### For NetworkManager:

```bash
# Restart interface
sudo nmcli device disconnect eth0
sudo nmcli device connect eth0

# Watch logs
sudo journalctl -u netevd -f
```

#### For dhclient:

```bash
# Restart dhclient
sudo systemctl restart dhclient

# Watch logs
sudo journalctl -u netevd -f
```

### Verify Routing Policy Rules

```bash
# Check if custom routing tables are created
ip rule list

# Should see rules like:
# 32765: from 192.168.1.100 lookup 203

# Check routing table
ip route show table 203
```

## Troubleshooting

### User Not Found Error

```
Error: Failed to drop privileges
Caused by: User 'netevd' not found
```

**Solution:** Create the netevd user (see [User Setup](#user-setup))

```bash
sudo useradd --system --no-create-home --shell /usr/sbin/nologin netevd
```

### Permission Denied Errors

```
Error: Permission denied (os error 13)
```

**Solution:** Ensure netevd has read access to configuration:

```bash
sudo chmod 644 /etc/netevd/netevd.yaml
sudo chmod 755 /etc/netevd
```

### Service Fails to Start

```bash
# Check detailed error
sudo systemctl status netevd
sudo journalctl -u netevd -n 50

# Common issues:
# 1. Configuration file syntax error
sudo /usr/bin/netevd  # Run manually to see error

# 2. Missing capabilities
sudo getcap /usr/bin/netevd
# Should show: cap_net_admin=eip (if using file capabilities)
```

### Scripts Not Executing

```bash
# Check script permissions
ls -la /etc/netevd/routable.d/

# Scripts must be executable
sudo chmod +x /etc/netevd/routable.d/*.sh

# Check logs for script execution
sudo journalctl -u netevd | grep "Executing"
```

### No Events Received

For systemd-networkd:
```bash
# Check if networkd is running
sudo systemctl status systemd-networkd

# Verify DBus is accessible
busctl list | grep networkd
```

For NetworkManager:
```bash
# Check if NetworkManager is running
sudo systemctl status NetworkManager

# Verify DBus
busctl list | grep NetworkManager
```

For dhclient:
```bash
# Check if lease file exists
ls -la /var/lib/dhclient/dhclient.leases

# Check if it's being modified
sudo inotifywait -m /var/lib/dhclient/dhclient.leases
```

### Debug Mode

Run with debug logging:

```bash
# Set environment variable
sudo RUST_LOG=debug systemctl restart netevd

# Or edit service file
sudo systemctl edit netevd

# Add:
[Service]
Environment="RUST_LOG=debug"

# Apply
sudo systemctl daemon-reload
sudo systemctl restart netevd
```

### Capabilities Issues

```bash
# Check capabilities
sudo getcap /usr/bin/netevd

# If using systemd, capabilities are granted by service file
cat /lib/systemd/system/netevd.service | grep Capabilit

# Should show:
# AmbientCapabilities=CAP_NET_ADMIN
# CapabilityBoundingSet=CAP_NET_ADMIN
```

## Uninstallation

To remove netevd:

```bash
# Stop and disable service
sudo systemctl stop netevd
sudo systemctl disable netevd

# Remove files
sudo rm /usr/bin/netevd
sudo rm /lib/systemd/system/netevd.service
sudo rm -rf /etc/netevd

# Remove user
sudo userdel netevd

# Reload systemd
sudo systemctl daemon-reload
```

## Security Notes

1. **User Isolation:** netevd runs as a dedicated system user, not root
2. **Capabilities:** Only CAP_NET_ADMIN is granted, not full root privileges
3. **Script Validation:** Environment variables are validated before passing to scripts
4. **No Network Access:** systemd service restricts network namespace if configured
5. **Read-only System:** systemd service mounts most of the filesystem read-only

## Next Steps

After installation:

1. Configure monitoring for your network manager
2. Create custom scripts for network events
3. Test routing policy rules if using multi-interface setup
4. Set up logging aggregation if needed
5. Configure monitoring/alerting for the service

## Support

- **Issues:** https://github.com/ssahani/netevd/issues
- **Documentation:** https://github.com/ssahani/netevd
- **Author:** Susant Sahani <ssahani@redhat.com>
