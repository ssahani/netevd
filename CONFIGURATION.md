<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Configuration Reference

netevd is configured via `/etc/netevd/netevd.yaml`. Changes require a service restart (`sudo systemctl restart netevd`) unless noted otherwise.

## Full Example

```yaml
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
  systemd_networkd:
    emit_json: true
  dhclient:
    use_dns: false
    use_domain: false
    use_hostname: false
  networkmanager: {}
```

## system

### log_level

Logging verbosity. Use `warn` or `info` in production, `debug` or `trace` for troubleshooting.

**Values:** `trace`, `debug`, `info`, `warn`, `error` | **Default:** `info`

Runtime override: `RUST_LOG=debug sudo systemctl restart netevd`

### backend

Which network manager to listen to for events.

**Values:** `systemd-networkd`, `NetworkManager`, `dhclient` | **Default:** `systemd-networkd`

| Backend | Best for | Notes |
|---------|----------|-------|
| `systemd-networkd` | Servers, containers | Lightweight, rich JSON data |
| `NetworkManager` | Desktops, laptops | Full-featured, GUI integration |
| `dhclient` | Legacy systems | Lease file monitoring via inotify |

## monitoring

### interfaces

Which interfaces to watch. An empty list monitors everything.

**Type:** Array of strings | **Default:** `[]` (all interfaces)

```yaml
monitoring:
  interfaces:
    - eth0
    - wlan0
```

## routing

### policy_rules

Interfaces that should get automatic routing policy rules. This solves multi-homed routing: traffic arriving on an interface will leave via the same interface.

**Type:** Array of strings | **Default:** `[]` (disabled)

For each listed interface, netevd creates:
- Custom routing table (ID = 200 + interface index)
- Source-based rule: `from <ip> lookup <table>`
- Destination-based rule: `to <ip> lookup <table>`
- Default route in the custom table via the interface's gateway

Rules are removed automatically when addresses are deleted.

```yaml
routing:
  policy_rules:
    - eth1      # Secondary interface gets its own routing table
    - eth2
```

## backends

### systemd_networkd

#### emit_json

Pass full interface data to scripts via the `$JSON` environment variable. Includes MTU, driver, addresses, DNS, routes, and more.

**Type:** Boolean | **Default:** `true`

Disable if your scripts don't use `$JSON` -- saves a small amount of overhead.

### dhclient

#### use_dns

Send DNS servers from DHCP lease to systemd-resolved. Requires systemd-resolved to be running.

**Type:** Boolean | **Default:** `false`

#### use_domain

Send domain name from DHCP lease to systemd-resolved.

**Type:** Boolean | **Default:** `false`

#### use_hostname

Send hostname from DHCP lease to systemd-hostnamed. **Warning:** this changes your system hostname.

**Type:** Boolean | **Default:** `false`

### networkmanager

No options currently. Placeholder for future configuration.

## Script Directories

Scripts live in `/etc/netevd/` under event-specific directories:

```
/etc/netevd/
├── netevd.yaml
├── carrier.d/          # Cable connected
├── no-carrier.d/       # Cable disconnected
├── configured.d/       # IP assigned (systemd-networkd)
├── degraded.d/         # Partial config (systemd-networkd)
├── routable.d/         # Full connectivity (systemd-networkd, dhclient)
├── activated.d/        # Device activated (NetworkManager)
├── disconnected.d/     # Device disconnected (NetworkManager)
├── manager.d/          # Manager state changes
└── routes.d/           # Route changes
```

**Rules:**
- Scripts must be executable (`chmod +x`)
- Scripts run in alphabetical order -- use `01-`, `02-` prefixes
- Must start with a shebang (`#!/bin/bash`)
- Non-zero exit codes are logged but don't stop other scripts

## Environment Variables

### All backends

| Variable | Description | Example |
|----------|-------------|---------|
| `LINK` | Interface name | `eth0` |
| `LINKINDEX` | Interface index | `2` |
| `STATE` | Current state | `routable` |
| `BACKEND` | Event source | `systemd-networkd` |
| `ADDRESSES` | Space-separated IPs | `192.168.1.100 10.0.0.5` |

### systemd-networkd only

| Variable | Description | Requires |
|----------|-------------|----------|
| `JSON` | Full interface data as JSON | `emit_json: true` |

### dhclient only

| Variable | Description |
|----------|-------------|
| `DHCP_ADDRESS` | Leased IP |
| `DHCP_GATEWAY` | Default gateway |
| `DHCP_DNS` | DNS servers |
| `DHCP_DOMAIN` | Domain name |
| `DHCP_HOSTNAME` | DHCP hostname |
| `DHCP_LEASE` | Full lease data |

## Configuration Profiles

### Laptop with WiFi

```yaml
system:
  log_level: "info"
  backend: "NetworkManager"
monitoring:
  interfaces:
    - wlan0
```

### Multi-NIC Server

```yaml
system:
  log_level: "warn"
  backend: "systemd-networkd"
monitoring:
  interfaces: [eth0, eth1, eth2]
routing:
  policy_rules: [eth1, eth2]
backends:
  systemd_networkd:
    emit_json: true
```

### VPN Gateway

```yaml
system:
  backend: "systemd-networkd"
monitoring:
  interfaces: [eth0, wg0]
routing:
  policy_rules: [wg0]
```

### DHCP with DNS Integration

```yaml
system:
  backend: "dhclient"
monitoring:
  interfaces: [eth0]
backends:
  dhclient:
    use_dns: true
    use_domain: true
```

## Validation

```bash
# Check YAML syntax
yamllint /etc/netevd/netevd.yaml

# Or with Python
python3 -c "import yaml; yaml.safe_load(open('/etc/netevd/netevd.yaml'))"

# Test with netevd
sudo netevd --config /etc/netevd/netevd.yaml --validate
```

## See Also

- [Quick Start](docs/QUICKSTART.md)
- [Examples](docs/EXAMPLES.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
