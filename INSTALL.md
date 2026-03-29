<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Installation

## Requirements

- Linux kernel 3.10+ with netlink support
- One of: systemd-networkd, NetworkManager, or dhclient
- **Build only:** Rust 1.70+, pkg-config, C compiler

## From Source (Recommended)

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build
git clone https://github.com/ssahani/netevd.git
cd netevd
cargo build --release

# Install
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service
sudo install -Dm644 examples/netevd.yaml /etc/netevd/netevd.yaml

# Create user and directories
sudo useradd -r -M -s /usr/bin/nologin -d /nonexistent netevd
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}

# Start
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
```

## Binary Release

```bash
# Download from GitHub releases (replace version)
wget https://github.com/ssahani/netevd/releases/download/vX.Y.Z/netevd-x86_64-unknown-linux-gnu.tar.gz
tar xzf netevd-x86_64-unknown-linux-gnu.tar.gz
sudo install -Dm755 netevd /usr/bin/netevd
```

Then install the service file, config, user, and directories as shown above.

## Package Managers

```bash
# crates.io
cargo install netevd

# Arch Linux (AUR)
yay -S netevd

# Fedora/RHEL
sudo dnf install netevd-X.Y.Z-1.x86_64.rpm

# Debian/Ubuntu
sudo dpkg -i netevd_X.Y.Z_amd64.deb
sudo apt-get install -f    # resolve dependencies if needed
```

## Post-Install Configuration

Edit `/etc/netevd/netevd.yaml` to match your setup:

```yaml
system:
  log_level: "info"
  backend: "systemd-networkd"    # or "NetworkManager" or "dhclient"

monitoring:
  interfaces:
    - eth0
    - eth1
```

Ensure your backend is running:

```bash
sudo systemctl status systemd-networkd    # or NetworkManager
```

## Verify

```bash
sudo systemctl status netevd
sudo journalctl -u netevd -f
netevd --version
```

## Upgrade

```bash
# From source
cd netevd && git pull && cargo build --release
sudo systemctl stop netevd
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo systemctl start netevd

# Package managers
yay -Syu netevd            # Arch
sudo dnf upgrade netevd     # Fedora
sudo apt-get upgrade netevd # Debian
```

## Uninstall

```bash
sudo systemctl stop netevd && sudo systemctl disable netevd
sudo rm /usr/bin/netevd /lib/systemd/system/netevd.service
sudo systemctl daemon-reload
sudo userdel netevd

# Optional: remove config (back up first)
sudo cp -r /etc/netevd /etc/netevd.backup
sudo rm -rf /etc/netevd
```

## Troubleshooting

**Service won't start:**
```bash
sudo systemctl status netevd -l
sudo journalctl -u netevd -n 50 --no-pager
```

Common causes: user `netevd` doesn't exist, YAML syntax error, missing capabilities.

**Build errors:**
```bash
rustup update stable
cargo clean && cargo build --release
```

See [Troubleshooting Guide](docs/TROUBLESHOOTING.md) for more.
