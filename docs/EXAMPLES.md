<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Real-World Examples

Step-by-step guides for common netevd use cases.

## Multi-Homed Server

**Problem:** Two interfaces in the same subnet. Traffic arriving on eth1 leaves via eth0 (default gateway), breaking return packets.

**Solution:** netevd creates per-interface routing tables automatically.

### Configuration

```yaml
# /etc/netevd/netevd.yaml
system:
  backend: "systemd-networkd"
monitoring:
  interfaces: [eth0, eth1]
routing:
  policy_rules:
    - eth1    # eth0 uses default table; eth1 gets its own
```

### Network setup (systemd-networkd)

```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0
[Network]
Address=192.168.1.10/24
Gateway=192.168.1.1
DNS=8.8.8.8

# /etc/systemd/network/20-eth1.network
[Match]
Name=eth1
[Network]
Address=192.168.1.11/24
# No default gateway -- netevd handles routing
```

### What happens

When eth1 (index 3) gets IP 192.168.1.11, netevd creates:

```bash
$ ip rule list
32765: from 192.168.1.11 lookup 203
32766: to 192.168.1.11 lookup 203

$ ip route show table 203
default via 192.168.1.1 dev eth1
```

### Verify

```bash
curl --interface eth1 https://ifconfig.me
ip route get 8.8.8.8 from 192.168.1.11    # should show "dev eth1 table 203"
```

---

## VPN Integration (WireGuard)

**Goal:** Automatically route private networks through VPN when it connects.

### Configuration

```yaml
system:
  backend: "systemd-networkd"
monitoring:
  interfaces: [eth0, wg0]
```

### Route script

```bash
#!/bin/bash
# /etc/netevd/routable.d/01-vpn-routes.sh

[ "$LINK" = "wg0" ] && [ "$STATE" = "routable" ] || exit 0

NETWORKS=("10.0.0.0/8" "172.16.0.0/12")

for net in "${NETWORKS[@]}"; do
    ip route add "$net" dev wg0 2>/dev/null
done

iptables -A FORWARD -i wg0 -j ACCEPT
iptables -A FORWARD -o wg0 -j ACCEPT

logger -t netevd-vpn "VPN routes configured via wg0"
```

### Cleanup script

```bash
#!/bin/bash
# /etc/netevd/no-carrier.d/01-vpn-cleanup.sh

[ "$LINK" = "wg0" ] || exit 0
logger -t netevd-vpn "VPN interface down, routes auto-cleaned by kernel"
```

```bash
sudo chmod +x /etc/netevd/routable.d/01-vpn-routes.sh
sudo chmod +x /etc/netevd/no-carrier.d/01-vpn-cleanup.sh
```

---

## High Availability Failover

**Goal:** Automatic failover from eth0 (primary) to eth1 (secondary) when primary loses carrier.

### Configuration

```yaml
system:
  backend: "systemd-networkd"
monitoring:
  interfaces: [eth0, eth1]
```

### Failover script

```bash
#!/bin/bash
# /etc/netevd/no-carrier.d/01-failover.sh

PRIMARY="eth0"
SECONDARY="eth1"
SECONDARY_GW="192.168.2.1"

[ "$LINK" = "$PRIMARY" ] || exit 0

if ip link show "$SECONDARY" | grep -q "state UP"; then
    ip route add default via "$SECONDARY_GW" dev "$SECONDARY" metric 100 2>/dev/null
    ip route flush cache
    logger -t netevd-ha "Failover to $SECONDARY"

    echo "Primary $PRIMARY failed on $(hostname). Failover active." | \
        mail -s "Network Failover Alert" admin@example.com
else
    logger -t netevd-ha "CRITICAL: Both interfaces down"
fi
```

### Recovery script

```bash
#!/bin/bash
# /etc/netevd/carrier.d/01-failback.sh

PRIMARY="eth0"
SECONDARY_GW="192.168.2.1"
PRIMARY_GW="192.168.1.1"

[ "$LINK" = "$PRIMARY" ] || exit 0

sleep 5    # Wait for stability
ip route del default via "$SECONDARY_GW" 2>/dev/null
ip route add default via "$PRIMARY_GW" dev "$PRIMARY" metric 50 2>/dev/null
ip route flush cache
logger -t netevd-ha "Recovered to primary $PRIMARY"
```

---

## Dynamic DNS Updates

**Goal:** Update DNS records when public IP changes.

### Cloudflare

```bash
#!/bin/bash
# /etc/netevd/routable.d/02-ddns-cloudflare.sh

[ "$LINK" = "eth0" ] || exit 0

ZONE_ID="your-zone-id"
RECORD_ID="your-record-id"
API_TOKEN="your-token"

IP=$(echo "$ADDRESSES" | grep -oE '([0-9]{1,3}\.){3}[0-9]{1,3}' | head -1)
[ -z "$IP" ] && exit 0

# Skip if IP hasn't changed
CACHE="/var/run/ddns-last-ip"
[ -f "$CACHE" ] && [ "$(cat "$CACHE")" = "$IP" ] && exit 0

RESPONSE=$(curl -s -X PUT \
    "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$RECORD_ID" \
    -H "Authorization: Bearer $API_TOKEN" \
    -H "Content-Type: application/json" \
    --data "{\"type\":\"A\",\"name\":\"home.example.com\",\"content\":\"$IP\",\"ttl\":120}")

if echo "$RESPONSE" | grep -q '"success":true'; then
    echo "$IP" > "$CACHE"
    logger -t netevd-ddns "Updated DNS: home.example.com -> $IP"
else
    logger -t netevd-ddns "DNS update failed: $RESPONSE"
    exit 1
fi
```

### DuckDNS (simpler)

```bash
#!/bin/bash
# /etc/netevd/routable.d/02-ddns-duckdns.sh

[ "$LINK" = "eth0" ] || exit 0
IP=$(echo "$ADDRESSES" | grep -oE '([0-9]{1,3}\.){3}[0-9]{1,3}' | head -1)
[ -z "$IP" ] && exit 0

curl -s "https://www.duckdns.org/update?domains=YOURDOMAIN&token=TOKEN&ip=$IP"
logger -t netevd-ddns "DuckDNS updated: $IP"
```

---

## Cloud Monitoring

### AWS CloudWatch

```bash
#!/bin/bash
# /etc/netevd/routable.d/03-cloudwatch.sh

aws cloudwatch put-metric-data \
    --namespace "NetEvd" \
    --metric-name InterfaceRouted \
    --value 1 \
    --dimensions Interface="$LINK",Host="$(hostname)"
```

### Datadog

```bash
#!/bin/bash
# /etc/netevd/routable.d/03-datadog.sh

curl -X POST "https://api.datadoghq.com/api/v1/events" \
    -H "DD-API-KEY: $DD_API_KEY" \
    -H "Content-Type: application/json" \
    -d "{
        \"title\": \"$LINK routable\",
        \"text\": \"$LINK on $(hostname): $ADDRESSES\",
        \"tags\": [\"interface:$LINK\", \"state:$STATE\"]
    }"
```

---

## Container Networking

```bash
#!/bin/bash
# /etc/netevd/routable.d/04-docker-routing.sh

[ "$LINK" = "eth1" ] && [ "$STATE" = "routable" ] || exit 0

ip route add 172.17.0.0/16 dev eth1 2>/dev/null
sysctl -w net.ipv4.ip_forward=1
iptables -t nat -A POSTROUTING -s 172.17.0.0/16 -o eth1 -j MASQUERADE
logger -t netevd "Docker routing via eth1"
```

---

## IoT Device Alerting

```bash
#!/bin/bash
# /etc/netevd/no-carrier.d/02-iot-alert.sh

[ "$LINK" = "eth2" ] || exit 0

curl -X POST https://monitoring.example.com/alerts \
    -H "Content-Type: application/json" \
    -d "{\"severity\":\"warning\",\"message\":\"IoT network $LINK down\",\"host\":\"$(hostname)\"}"
```

---

## JSON Processing (systemd-networkd)

```bash
#!/bin/bash
# /etc/netevd/routable.d/03-json-processing.sh

[ -z "$JSON" ] && exit 0

MTU=$(echo "$JSON" | jq -r '.MTU')
DRIVER=$(echo "$JSON" | jq -r '.Driver')

logger -t netevd "$LINK: MTU=$MTU, Driver=$DRIVER"

# Auto-fix oversized MTU
if [ "$MTU" -gt 1500 ]; then
    ip link set dev "$LINK" mtu 1500
    logger -t netevd "Adjusted $LINK MTU to 1500"
fi
```

---

## Conditional Execution

```bash
#!/bin/bash
# /etc/netevd/routable.d/01-conditional.sh

# Only during business hours
HOUR=$(date +%H)
[ "$HOUR" -lt 9 ] || [ "$HOUR" -gt 17 ] && exit 0

# Only for specific interfaces
case "$LINK" in
    eth0|eth1) ;;
    *) exit 0 ;;
esac

# Your logic here
logger -t netevd "Processed $LINK during business hours"
```

## See Also

- [Configuration](../CONFIGURATION.md) -- all config options
- [Quick Start](QUICKSTART.md) -- getting started
- [Troubleshooting](TROUBLESHOOTING.md) -- when things go wrong
