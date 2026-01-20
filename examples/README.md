# netevd Examples

This directory contains example configurations and scripts for netevd.

## Configuration Example

- **netevd.yaml** - Example configuration file showing all available options

### Installation

```bash
sudo mkdir -p /etc/netevd
sudo install -Dm644 netevd.yaml /etc/netevd/netevd.yaml
```

Edit `/etc/netevd/netevd.yaml` to customize for your environment.

## Script Examples

Example scripts demonstrating how to manually manage routing policy rules:

- **ip_routing_rule_add.sh** - Add routing policy rules manually
- **ip_routing_rule_remove.sh** - Remove routing policy rules manually

These scripts show the underlying `ip rule` and `ip route` commands that netevd uses automatically.

## Script Directories

When netevd is running, place your custom scripts in `/etc/netevd/`:

```
/etc/netevd/
├── netevd.yaml              # Main configuration
├── carrier.d/               # Link has carrier (cable connected)
├── no-carrier.d/            # Link lost carrier (cable disconnected)
├── configured.d/            # Link is configured
├── degraded.d/              # Link is degraded
├── routable.d/              # Link is routable (has working network)
├── activated.d/             # Device activated (NetworkManager)
├── disconnected.d/          # Device disconnected (NetworkManager)
├── manager.d/               # Network manager state changes
└── routes.d/                # Route changes detected
```

## Example Scripts for Network Events

### Routable Event Example

Create `/etc/netevd/routable.d/01-notify.sh`:

```bash
#!/bin/bash
# Runs when interface becomes routable

echo "Interface $LINK is now routable"
echo "IP Addresses: $ADDRESSES"

# Send notification
notify-send "Network Ready" "Interface $LINK: $ADDRESSES"

# Start services that depend on network
systemctl start myapp.service
```

Make it executable:
```bash
sudo chmod +x /etc/netevd/routable.d/01-notify.sh
```

### Route Change Example

Create `/etc/netevd/routes.d/01-log-routes.sh`:

```bash
#!/bin/bash
# Runs when routes change

logger -t netevd "Route $EVENT on interface $LINK ($LINKINDEX)"

# Optionally log to file
echo "$(date): Route $EVENT on $LINK" >> /var/log/netevd-routes.log
```

### Carrier Loss Example

Create `/etc/netevd/no-carrier.d/01-alert.sh`:

```bash
#!/bin/bash
# Runs when interface loses carrier (cable unplugged)

# Send email alert
echo "Interface $LINK lost carrier at $(date)" | \
    mail -s "Network Alert: Link Down" admin@example.com

# Log to syslog
logger -t netevd -p daemon.warning "Link $LINK carrier lost"
```

## Available Environment Variables

Scripts receive these environment variables:

### Common (All Backends)
- `LINK` - Interface name (e.g., `eth0`)
- `LINKINDEX` - Interface index number
- `STATE` - Current state (e.g., `routable`, `activated`)
- `EVENT` - Event type for route changes (`new`, `del`)
- `ADDRESSES` - Space-separated list of IP addresses

### systemd-networkd Specific
- `JSON` - Full interface data in JSON format (if `emit_json: true`)

### dhclient Specific
- `DHCP_ADDRESS` - IP address from DHCP lease
- `DHCP_GATEWAY` - Default gateway
- `DHCP_DNS` - DNS servers
- `DHCP_DOMAIN` - Domain name
- `DHCP_HOSTNAME` - Hostname from DHCP

## Testing Scripts

Test your scripts manually:

```bash
# Set environment and run script
sudo env LINK=eth0 LINKINDEX=2 STATE=routable \
     ADDRESSES="192.168.1.100" \
     /etc/netevd/routable.d/01-notify.sh
```

## See Also

- [Main README](../README.md) - Full documentation
- [INSTALL.md](../INSTALL.md) - Installation guide
- [systemd/](../systemd/) - Systemd service files
