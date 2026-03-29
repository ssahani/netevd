<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Quick Start

Get netevd running in 5 minutes. By the end, you'll have a daemon watching your network interfaces and executing custom scripts on state changes.

## What netevd Does

```
Network change  --->  netevd detects it  --->  Your script runs
(IP added,            (via netlink /           (update DNS,
 link down,            DBus / inotify)          send alert,
 route changed)                                 configure VPN...)
```

## Step 1: Install

**From source (recommended):**

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
```

**From package:**

```bash
yay -S netevd                          # Arch Linux
sudo dnf install netevd-*.rpm          # Fedora/RHEL
sudo dpkg -i netevd_*.deb             # Debian/Ubuntu
```

## Step 2: Create User and Directories

```bash
sudo useradd -r -M -s /usr/bin/nologin -d /nonexistent netevd
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}
```

## Step 3: Write Your First Script

```bash
sudo tee /etc/netevd/routable.d/01-notify.sh > /dev/null << 'EOF'
#!/bin/bash
logger -t netevd "$(date): $LINK is routable with $ADDRESSES"
EOF
sudo chmod +x /etc/netevd/routable.d/01-notify.sh
```

## Step 4: Start

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
sudo systemctl status netevd
```

## Step 5: Test

```bash
# Trigger an event
sudo ip link set eth0 down && sudo ip link set eth0 up

# Check it worked
sudo journalctl -t netevd -n 10
```

You should see your script's output in the logs.

## What Variables Can Scripts Use?

| Variable | Example | Description |
|----------|---------|-------------|
| `$LINK` | `eth0` | Interface name |
| `$LINKINDEX` | `2` | Kernel interface index |
| `$STATE` | `routable` | Current state |
| `$BACKEND` | `systemd-networkd` | Which network manager |
| `$ADDRESSES` | `192.168.1.100 10.0.0.5` | Space-separated IPs |
| `$JSON` | `{"Index":2,...}` | Full data (systemd-networkd only) |

## Script Directories

| Directory | When scripts run |
|-----------|-----------------|
| `carrier.d/` | Cable connected |
| `no-carrier.d/` | Cable disconnected |
| `routable.d/` | Interface has full connectivity |
| `configured.d/` | Interface has IP (systemd-networkd) |
| `degraded.d/` | Partial config (systemd-networkd) |
| `activated.d/` | Device activated (NetworkManager) |
| `disconnected.d/` | Device disconnected (NetworkManager) |
| `routes.d/` | Routing table changed |

Scripts run alphabetically. Use `01-`, `02-` prefixes to control order.

## More Examples

**Alert on link down:**

```bash
sudo tee /etc/netevd/no-carrier.d/01-alert.sh > /dev/null << 'EOF'
#!/bin/bash
echo "ALERT: $LINK lost carrier at $(date)" | \
    mail -s "Network Alert: Link Down" admin@example.com
EOF
sudo chmod +x /etc/netevd/no-carrier.d/01-alert.sh
```

**Update dynamic DNS:**

```bash
sudo tee /etc/netevd/routable.d/02-ddns.sh > /dev/null << 'EOF'
#!/bin/bash
[ "$LINK" = "eth0" ] || exit 0
IP=$(echo "$ADDRESSES" | awk '{print $1}')
curl -s "https://www.duckdns.org/update?domains=YOURDOMAIN&token=TOKEN&ip=$IP"
logger -t netevd "Updated DNS: $IP"
EOF
sudo chmod +x /etc/netevd/routable.d/02-ddns.sh
```

## Switching Backends

The default backend is systemd-networkd. To use a different one, edit `/etc/netevd/netevd.yaml`:

```yaml
system:
  backend: "NetworkManager"    # or "dhclient"
```

Then restart: `sudo systemctl restart netevd`

## Debugging

```bash
sudo journalctl -u netevd -f                        # Follow logs
sudo journalctl -u netevd | grep "Executing"         # Find script runs

# Test a script manually
sudo env LINK=eth0 LINKINDEX=2 STATE=routable \
    ADDRESSES="192.168.1.100" \
    /etc/netevd/routable.d/01-notify.sh

# Enable verbose logging
sudo sed -i 's/log_level: "info"/log_level: "debug"/' /etc/netevd/netevd.yaml
sudo systemctl restart netevd
```

## Next Steps

- [Configuration Guide](../CONFIGURATION.md) -- All YAML options
- [Real-World Examples](EXAMPLES.md) -- Multi-homing, VPN, HA, containers
- [REST API](API.md) -- Remote management via HTTP
- [Prometheus Metrics](METRICS.md) -- Monitoring and alerting
- [Troubleshooting](TROUBLESHOOTING.md) -- Common issues and fixes
