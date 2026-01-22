<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Installation Guide

This guide covers various methods to install netevd on your system.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation Methods](#installation-methods)
  - [From Source](#from-source)
  - [Binary Release](#binary-release)
  - [Package Managers](#package-managers)
- [Post-Installation Setup](#post-installation-setup)
- [Verification](#verification)
- [Uninstallation](#uninstallation)

## System Requirements

### Operating System
- Linux with kernel 3.10 or later
- systemd (for systemd-networkd backend and service management)
- DBus support

### Dependencies

#### Runtime Dependencies
- `systemd` (recommended)
- `systemd-networkd` or `NetworkManager` or `dhclient`
- Linux kernel with netlink support

#### Build Dependencies
- Rust 1.70 or later
- Cargo
- GCC or Clang (for linking)
- pkg-config
- Development headers for system libraries

## Installation Methods

### From Source

This is the recommended method for most users and provides the latest features.

#### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

#### 2. Clone the Repository

```bash
git clone https://github.com/ssahani/netevd.git
cd netevd
```

#### 3. Build

```bash
# Build in release mode (optimized)
cargo build --release

# The binary will be at: target/release/netevd
```

#### 4. Install System-wide

```bash
# Install binary
sudo install -Dm755 target/release/netevd /usr/bin/netevd

# Install systemd service file
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service

# Install example configuration
sudo install -Dm644 examples/netevd.yaml /etc/netevd/netevd.yaml
```

#### 5. Create Required Directories

```bash
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}
```

#### 6. Create System User

```bash
sudo useradd -r -M -s /usr/bin/nologin -d /nonexistent netevd
```

#### 7. Enable and Start Service

```bash
sudo systemctl daemon-reload
sudo systemctl enable netevd
sudo systemctl start netevd
```

### Binary Release

Download pre-built binaries from the [releases page](https://github.com/ssahani/netevd/releases).

```bash
# Download latest release (replace X.Y.Z with actual version)
wget https://github.com/ssahani/netevd/releases/download/vX.Y.Z/netevd-x86_64-unknown-linux-gnu.tar.gz

# Extract
tar xzf netevd-x86_64-unknown-linux-gnu.tar.gz

# Install
sudo install -Dm755 netevd /usr/bin/netevd
```

Then follow steps 4-7 from the "From Source" section above.

### Package Managers

#### Cargo (crates.io)

```bash
cargo install netevd
```

The binary will be installed to `~/.cargo/bin/netevd`. You'll still need to:
- Copy it to `/usr/bin/` (or add `~/.cargo/bin` to PATH)
- Install the systemd service file manually
- Create configuration and directories

#### Arch Linux (AUR)

```bash
# Using yay
yay -S netevd

# Using paru
paru -S netevd

# Manual
git clone https://aur.archlinux.org/netevd.git
cd netevd
makepkg -si
```

#### Fedora/RHEL/CentOS

```bash
# Download RPM from releases
sudo dnf install netevd-X.Y.Z-1.x86_64.rpm

# Or using rpm directly
sudo rpm -ivh netevd-X.Y.Z-1.x86_64.rpm
```

#### Debian/Ubuntu

```bash
# Download DEB from releases
sudo dpkg -i netevd_X.Y.Z_amd64.deb

# Install dependencies if needed
sudo apt-get install -f
```

## Post-Installation Setup

### 1. Configure netevd

Edit `/etc/netevd/netevd.yaml`:

```yaml
system:
  log_level: "info"
  backend: "systemd-networkd"  # or "NetworkManager" or "dhclient"

network:
  links: "eth0 eth1"  # Space-separated list of interfaces to monitor
  routing_policy_rules: ""  # Interfaces needing custom routing
  emit_json: true
  use_dns: false
  use_domain: false
  use_hostname: false
```

### 2. Set Up Scripts (Optional)

Create executable scripts in the appropriate directories:

```bash
# Example: Create a script that runs when interface is routable
sudo cat > /etc/netevd/routable.d/01-notify.sh << 'EOF'
#!/bin/bash
echo "Interface $LINK is now routable with IP: $ADDRESSES"
logger -t netevd "Interface $LINK is routable"
EOF

sudo chmod +x /etc/netevd/routable.d/01-notify.sh
```

### 3. Configure Your Network Backend

#### For systemd-networkd

Ensure systemd-networkd is running:

```bash
sudo systemctl enable systemd-networkd
sudo systemctl start systemd-networkd
```

#### For NetworkManager

Ensure NetworkManager is running:

```bash
sudo systemctl enable NetworkManager
sudo systemctl start NetworkManager
```

Configure netevd to use NetworkManager:

```yaml
system:
  backend: "NetworkManager"
```

#### For dhclient

Install dhclient if not present:

```bash
# Debian/Ubuntu
sudo apt-get install isc-dhcp-client

# Fedora/RHEL
sudo dnf install dhclient
```

Configure netevd:

```yaml
system:
  backend: "dhclient"

network:
  use_dns: true
  use_domain: true
  use_hostname: true
```

### 4. Adjust Permissions (if needed)

```bash
# Ensure netevd user can read configuration
sudo chown -R root:netevd /etc/netevd
sudo chmod -R 750 /etc/netevd
```

## Verification

### Check Service Status

```bash
sudo systemctl status netevd
```

Expected output:
```
● netevd.service - Network Event Daemon
     Loaded: loaded (/lib/systemd/system/netevd.service; enabled; vendor preset: enabled)
     Active: active (running) since ...
```

### View Logs

```bash
# Follow logs in real-time
sudo journalctl -u netevd -f

# View recent logs
sudo journalctl -u netevd -n 50
```

### Test Network Events

Trigger a network event and check if netevd responds:

```bash
# For systemd-networkd
sudo networkctl reload

# For NetworkManager
sudo nmcli device disconnect eth0
sudo nmcli device connect eth0

# Check logs
sudo journalctl -u netevd -n 20
```

### Verify Binary

```bash
# Check version
netevd --version

# Verify installation path
which netevd

# Check file permissions
ls -la /usr/bin/netevd
```

## Troubleshooting Installation

### Service Fails to Start

```bash
# Check detailed status
sudo systemctl status netevd -l

# Check for errors
sudo journalctl -u netevd -n 100 --no-pager
```

Common issues:
- **User doesn't exist**: Run `sudo useradd -r -M -s /usr/bin/nologin netevd`
- **Configuration errors**: Validate YAML syntax in `/etc/netevd/netevd.yaml`
- **Permission denied**: Check file ownership and permissions

### Build Errors

```bash
# Update Rust
rustup update stable

# Clean build
cargo clean
cargo build --release

# Check for missing dependencies
pkg-config --list-all | grep -i ssl
```

### Missing Capabilities

If netevd can't configure network:

```bash
# Option 1: Set file capabilities (systemd manages this)
sudo setcap cap_net_admin+eip /usr/bin/netevd

# Option 2: Use AmbientCapabilities in systemd service (preferred)
# Already configured in netevd.service
```

## Uninstallation

### Stop and Disable Service

```bash
sudo systemctl stop netevd
sudo systemctl disable netevd
```

### Remove Files

```bash
# Remove binary
sudo rm /usr/bin/netevd

# Remove systemd service
sudo rm /lib/systemd/system/netevd.service
sudo systemctl daemon-reload

# Remove configuration (optional - backup first!)
sudo cp -r /etc/netevd /etc/netevd.backup
sudo rm -rf /etc/netevd

# Remove user
sudo userdel netevd
```

### Package Manager Uninstall

```bash
# Arch Linux
yay -R netevd

# Fedora/RHEL
sudo dnf remove netevd

# Debian/Ubuntu
sudo apt-get remove netevd

# Cargo
cargo uninstall netevd
```

## Upgrading

### From Source

```bash
cd netevd
git pull origin main
cargo build --release
sudo systemctl stop netevd
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo systemctl start netevd
```

### Using Package Manager

```bash
# Arch Linux
yay -Syu netevd

# Fedora/RHEL
sudo dnf upgrade netevd

# Debian/Ubuntu
sudo apt-get update
sudo apt-get upgrade netevd
```

## Next Steps

After installation:
1. Read [CONFIGURATION.md](CONFIGURATION.md) for detailed configuration options
2. Check [README.md](README.md) for usage examples
3. Review [SECURITY.md](SECURITY.md) for security best practices
4. See [CONTRIBUTING.md](CONTRIBUTING.md) if you want to contribute

## Support

- GitHub Issues: https://github.com/ssahani/netevd/issues
- Documentation: https://github.com/ssahani/netevd
