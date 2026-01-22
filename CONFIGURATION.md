<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Configuration Guide

This guide provides detailed information about configuring netevd for your specific needs.

## Table of Contents

- [Configuration File](#configuration-file)
- [System Section](#system-section)
- [Network Section](#network-section)
- [Script Directories](#script-directories)
- [Environment Variables](#environment-variables)
- [Advanced Configuration](#advanced-configuration)
- [Examples](#examples)

## Configuration File

The main configuration file is located at `/etc/netevd/netevd.yaml` and uses YAML format.

### Basic Structure

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

### Configuration Validation

Validate your configuration before applying:

```bash
# Check YAML syntax
yamllint /etc/netevd/netevd.yaml

# Test configuration with netevd
sudo netevd --config /etc/netevd/netevd.yaml --validate
```

## System Section

Controls general daemon behavior and backend selection.

### log_level

**Type:** String
**Default:** `"info"`
**Values:** `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`

Controls logging verbosity.

```yaml
system:
  log_level: "info"
```

**Recommended settings:**
- Production: `"warn"` or `"info"`
- Development: `"debug"`
- Troubleshooting: `"trace"`

**Runtime override:**
```bash
RUST_LOG=debug sudo systemctl restart netevd
```

### backend

**Type:** String
**Default:** `"systemd-networkd"`
**Values:** `"systemd-networkd"`, `"NetworkManager"`, `"dhclient"`

Specifies which network management system to monitor.

```yaml
system:
  backend: "systemd-networkd"
```

**Backend comparison:**

| Backend | Best For | Pros | Cons |
|---------|----------|------|------|
| `systemd-networkd` | Servers, containers | Fast, lightweight, rich JSON | systemd required |
| `NetworkManager` | Desktops, laptops | Full-featured, GUI integration | Heavier |
| `dhclient` | Legacy systems | Widely compatible | Limited features |

## Monitoring Section

Controls which network interfaces to monitor.

### interfaces

**Type:** Array of strings
**Default:** Empty (monitor all interfaces)
**Format:** YAML array

Specifies which interfaces to monitor.

```yaml
monitoring:
  interfaces:
    - eth0
    - eth1
    - wlan0
```

**Special values:**
- Empty array `[]`: Monitor all interfaces
- Single interface: List with one item
- Multiple: List with multiple items

**Examples:**
```yaml
# Monitor all interfaces
monitoring:
  interfaces: []

# Monitor only specific interfaces
monitoring:
  interfaces:
    - eth0
    - eth1

# Monitor wireless only
monitoring:
  interfaces:
    - wlan0
```

## Routing Section

Controls routing policy rule creation.

### policy_rules

**Type:** Array of strings
**Default:** Empty (disabled)
**Format:** YAML array of interface names

Enables automatic routing policy rule creation for specified interfaces.

```yaml
routing:
  policy_rules:
    - eth1
    - eth2
```

**What it does:**

For each interface, netevd automatically creates:

1. **Custom routing table** (ID = 200 + interface index)
2. **Source-based rule** (`from <ip> lookup <table>`)
3. **Destination-based rule** (`to <ip> lookup <table>`)
4. **Default route** in custom table

**Use cases:**
- Multi-homed servers
- Secondary interfaces in same subnet
- Load balancing across multiple connections
- VPN and regular traffic separation

**Example scenario:**

```yaml
routing:
  policy_rules:
    - eth1
```

When eth1 (index 3) gets IP 192.168.1.100:

```bash
# Rules created automatically:
ip rule add from 192.168.1.100 lookup 203
ip rule add to 192.168.1.100 lookup 203
ip route add default via 192.168.1.1 dev eth1 table 203
```

## Backends Section

Backend-specific configuration options.

### systemd_networkd

Configuration for systemd-networkd backend.

#### emit_json

**Type:** Boolean
**Default:** `true`

Controls whether JSON data is passed to scripts via `$JSON` environment variable.

```yaml
backends:
  systemd_networkd:
    emit_json: true
```

**When enabled**, scripts receive rich interface data:
```bash
#!/bin/bash
# Access JSON data
echo "$JSON" | jq '.MTU'
echo "$JSON" | jq '.Driver'
echo "$JSON" | jq '.Address[].IP'
```

**Performance note:** Disable if you don't need JSON to reduce overhead.

### dhclient

Configuration for dhclient backend.

#### use_dns

**Type:** Boolean
**Default:** `false`

Send DNS servers from DHCP to systemd-resolved.

```yaml
backends:
  dhclient:
    use_dns: true
```

**Requires:**
- dhclient backend
- systemd-resolved running

#### use_domain

**Type:** Boolean
**Default:** `false`

Send domain name from DHCP to systemd-resolved.

```yaml
backends:
  dhclient:
    use_domain: true
```

#### use_hostname

**Type:** Boolean
**Default:** `false`

Send hostname from DHCP to systemd-hostnamed.

```yaml
backends:
  dhclient:
    use_hostname: true
```

**Warning:** This will change your system hostname based on DHCP response.

### networkmanager

Configuration for NetworkManager backend.

```yaml
backends:
  networkmanager: {}
```

*Currently no specific options. Placeholder for future features.*

## Script Directories

Scripts are organized by event type in `/etc/netevd/`.

### Directory Structure

```
/etc/netevd/
├── carrier.d/          # Link has carrier (cable connected)
├── no-carrier.d/       # Link lost carrier (cable disconnected)
├── configured.d/       # Link is configured (systemd-networkd)
├── degraded.d/         # Link is degraded (systemd-networkd)
├── routable.d/         # Link is routable (has working network)
├── activated.d/        # Device activated (NetworkManager)
├── disconnected.d/     # Device disconnected (NetworkManager)
├── manager.d/          # Network manager state changes
└── routes.d/           # Route changes detected
```

### Script Requirements

1. **Must be executable**
   ```bash
   chmod +x /etc/netevd/routable.d/01-script.sh
   ```

2. **Naming convention** (optional but recommended)
   - Use numeric prefixes for ordering: `01-`, `02-`, `03-`
   - Scripts run in alphabetical order
   - Example: `01-update-dns.sh`, `02-notify.sh`

3. **Shebang required**
   ```bash
   #!/bin/bash
   ```

4. **Exit codes**
   - 0: Success
   - Non-zero: Error (logged but doesn't stop other scripts)

### Event-to-Directory Mapping

| Event | Directory | Backend | Trigger |
|-------|-----------|---------|---------|
| Link carrier gained | `carrier.d/` | All | Physical link up |
| Link carrier lost | `no-carrier.d/` | All | Cable unplugged |
| Interface configured | `configured.d/` | systemd-networkd | IP configured |
| Interface degraded | `degraded.d/` | systemd-networkd | Partial config |
| Interface routable | `routable.d/` | systemd-networkd, dhclient | Gateway reachable |
| Device activated | `activated.d/` | NetworkManager | Connection active |
| Device disconnected | `disconnected.d/` | NetworkManager | Connection down |
| Route changed | `routes.d/` | All | Kernel route update |

## Environment Variables

All scripts receive these environment variables.

### Common Variables (All Backends)

| Variable | Description | Example |
|----------|-------------|---------|
| `LINK` | Interface name | `eth0` |
| `LINKINDEX` | Interface index | `2` |
| `STATE` | Current state | `routable` |
| `BACKEND` | Event source | `systemd-networkd` |
| `ADDRESSES` | Space-separated IPs | `192.168.1.100 10.0.0.5` |

### systemd-networkd Specific

| Variable | Description | Requires |
|----------|-------------|----------|
| `JSON` | Full interface JSON | `emit_json: true` |

**JSON structure:**
```json
{
  "Index": 2,
  "Name": "eth0",
  "OperState": "up",
  "MTU": 1500,
  "Driver": "e1000e",
  "IPv4AddressState": "routable",
  "Address": [
    {"IP": "192.168.1.100", "Mask": 24}
  ],
  "DNS": ["8.8.8.8", "8.8.4.4"],
  "Gateway": "192.168.1.1"
}
```

### dhclient Specific

| Variable | Description | Example |
|----------|-------------|---------|
| `DHCP_ADDRESS` | Leased IP | `192.168.1.100` |
| `DHCP_GATEWAY` | Default gateway | `192.168.1.1` |
| `DHCP_DNS` | DNS servers | `8.8.8.8 8.8.4.4` |
| `DHCP_DOMAIN` | Domain name | `example.com` |
| `DHCP_HOSTNAME` | DHCP hostname | `myhost` |
| `DHCP_LEASE` | Full lease info | (varies) |

## Advanced Configuration

### Multiple Interface Routing

Configure multiple interfaces with custom routing:

```yaml
system:
  backend: "systemd-networkd"

monitoring:
  interfaces:
    - eth0
    - eth1
    - eth2

routing:
  policy_rules:  # eth0 is default gateway
    - eth1
    - eth2
```

**Result:**
- eth0: Uses main routing table (default route)
- eth1: Custom table 203, source-based routing
- eth2: Custom table 204, source-based routing

### Conditional Script Execution

Scripts can check variables to run conditionally:

```bash
#!/bin/bash
# Only run for specific interface
if [ "$LINK" != "eth0" ]; then
    exit 0
fi

# Only run for specific backend
if [ "$BACKEND" != "systemd-networkd" ]; then
    exit 0
fi

# Your logic here
echo "Processing eth0 from systemd-networkd"
```

### Dynamic Configuration

Scripts can modify behavior based on configuration:

```bash
#!/bin/bash
# Read custom config
source /etc/netevd/custom.conf

if [ "$ENABLE_FEATURE" = "true" ]; then
    # Feature logic
fi
```

### Integration with systemd

Create drop-in configuration for systemd service:

```bash
sudo systemctl edit netevd
```

Add custom settings:
```ini
[Service]
Environment="RUST_LOG=debug"
Environment="CUSTOM_VAR=value"
CPUQuota=50%
MemoryLimit=100M
```

### Logging Configuration

Control where logs go:

```bash
# View logs
sudo journalctl -u netevd -f

# Save to file
sudo journalctl -u netevd > /var/log/netevd.log

# Rotate logs (systemd does this automatically)
sudo journalctl --vacuum-time=7d
```

## Examples

### Example 1: Laptop with WiFi

```yaml
system:
  log_level: "info"
  backend: "NetworkManager"

monitoring:
  interfaces:
    - wlan0
```

### Example 2: Server with Multiple NICs

```yaml
system:
  log_level: "warn"
  backend: "systemd-networkd"

monitoring:
  interfaces:
    - eno1
    - eno2
    - eno3
    - eno4

routing:
  policy_rules:
    - eno2
    - eno3
    - eno4

backends:
  systemd_networkd:
    emit_json: true
```

### Example 3: VPN Gateway

```yaml
system:
  log_level: "info"
  backend: "systemd-networkd"

monitoring:
  interfaces:
    - eth0
    - wg0

routing:
  policy_rules:
    - wg0

backends:
  systemd_networkd:
    emit_json: true
```

With script `/etc/netevd/routable.d/01-vpn-routes.sh`:
```bash
#!/bin/bash
if [ "$LINK" = "wg0" ]; then
    ip route add 10.0.0.0/8 dev wg0
    logger "VPN routes configured"
fi
```

### Example 4: DHCP with DNS Integration

```yaml
system:
  log_level: "info"
  backend: "dhclient"

monitoring:
  interfaces:
    - eth0

backends:
  dhclient:
    use_dns: true
    use_domain: true
    use_hostname: false
```

### Example 5: High-Availability Setup

```yaml
system:
  log_level: "debug"
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
```

With failover script `/etc/netevd/no-carrier.d/01-failover.sh`:
```bash
#!/bin/bash
PRIMARY="eth0"
SECONDARY="eth1"

if [ "$LINK" = "$PRIMARY" ]; then
    # Primary failed, ensure secondary takes over
    logger "Primary link $PRIMARY down, checking $SECONDARY"
    ip route add default via 192.168.2.1 dev $SECONDARY metric 100
fi
```

## Configuration Best Practices

1. **Start simple**: Begin with minimal configuration and add features as needed
2. **Test scripts**: Always test scripts manually before adding to production
3. **Use version control**: Keep configuration in git
4. **Monitor logs**: Regularly check logs for errors
5. **Document changes**: Comment your scripts and configuration
6. **Backup configuration**: Before making changes
   ```bash
   sudo cp -r /etc/netevd /etc/netevd.backup
   ```
7. **Use validation**: Test configuration before restarting service
8. **Follow naming conventions**: Use consistent script naming

## Troubleshooting Configuration

### Configuration Not Loading

```bash
# Check syntax
yamllint /etc/netevd/netevd.yaml

# Check permissions
ls -la /etc/netevd/netevd.yaml

# Should be readable by netevd user
sudo chmod 644 /etc/netevd/netevd.yaml
```

### Scripts Not Running

```bash
# Check execute permissions
ls -la /etc/netevd/routable.d/

# Make executable
sudo chmod +x /etc/netevd/routable.d/*.sh

# Test manually
sudo env LINK=eth0 STATE=routable /etc/netevd/routable.d/01-test.sh
```

### Routing Rules Not Created

```bash
# Verify configuration
grep routing_policy_rules /etc/netevd/netevd.yaml

# Check interface is monitored
grep links /etc/netevd/netevd.yaml

# View logs for errors
sudo journalctl -u netevd | grep -i "routing"
```

## See Also

- [README.md](README.md) - Project overview and quick start
- [INSTALL.md](INSTALL.md) - Installation instructions
- [SECURITY.md](SECURITY.md) - Security considerations
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
