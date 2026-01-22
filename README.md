# netevd - Network Event Daemon

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)

A high-performance network event daemon written in Rust that configures network interfaces and executes scripts on network events from systemd-networkd, NetworkManager DBus signals, or when dhclient gains a lease. It also monitors:

1. Address changes (added/removed/modified)
2. Link changes (added/removed)
3. Route modifications

## ✨ Features

### Core Features
- 🚀 **Async/Await Architecture**: Built on tokio for efficient event handling
- 🔌 **Multiple Network Managers**: Supports systemd-networkd, NetworkManager, and dhclient
- 🛣️ **Routing Policy Rules**: Automatically configures multi-interface routing with custom routing tables
- 📜 **Script Execution**: Executes user-defined scripts on network state changes
- 🔒 **Security**: Runs as unprivileged user with minimal capabilities (CAP_NET_ADMIN only)
- ⚡ **Real-time Monitoring**: Sub-100ms event latency via netlink multicast
- 🎯 **Input Validation**: Defense-in-depth against command injection
- 🔄 **Atomic State Updates**: Race-free network state management

### New in v0.2.0 🎉

#### Developer Tools
- 🖥️ **Enhanced CLI**: Comprehensive command-line interface with `status`, `list`, `show`, `events`, `reload`, `validate`, and `test` commands
- ✅ **Config Validation**: Built-in YAML configuration validation tool
- 🧪 **Dry-run Mode**: Test configuration changes safely without applying them
- 📊 **Multiple Output Formats**: JSON, YAML, and table formats for all commands

#### Enterprise Features
- 🌐 **REST API**: Full-featured HTTP API built with Axum framework (9 endpoints)
- 🔍 **Event Filtering**: Advanced event filtering with pattern matching and conditional expressions
- 📈 **Prometheus Metrics**: 15+ metrics across 6 categories for comprehensive monitoring
- 📝 **Audit Logging**: Structured JSON audit logs for compliance and debugging
- 🌍 **IPv6 Support**: Policy routing for IPv6 with RFC 6724 address selection
- 🎨 **Web Dashboard**: Real-time monitoring interface with auto-refresh

#### Cloud & Kubernetes
- ☸️ **Kubernetes Operator**: Custom Resource Definitions and DaemonSet deployment
- 🐳 **Docker Images**: Debian (~150MB) and Alpine (~50MB) container images
- ☁️ **Cloud Provider Integration**: AWS EC2, Azure, and GCP API integrations
- 📦 **Multiple Distribution Formats**: Available on crates.io, with RPM, DEB, and AUR packages

## 📊 Architecture Overview

```mermaid
graph TB
    subgraph "Network Backends"
        NM[NetworkManager<br/>DBus Signals]
        SN[systemd-networkd<br/>DBus Signals]
        DC[dhclient<br/>Lease File]
    end

    subgraph "netevd Core"
        ML[Main Loop<br/>tokio::select!]

        subgraph "Event Sources"
            BL[Backend Listener<br/>DBus/FileWatch]
            AW[Address Watcher<br/>Netlink Events]
            RW[Route Watcher<br/>Netlink Events]
            LW[Link Watcher<br/>Netlink Events]
        end

        subgraph "State Management"
            NS[NetworkState<br/>Arc RwLock]
            RL[Routing Logic]
        end

        subgraph "Actions"
            SE[Script Executor<br/>Input Validation]
            RT[Route Tables<br/>ip rule/route]
            BUS[DBus Services<br/>resolved/hostnamed]
        end
    end

    subgraph "User Scripts"
        S1[carrier.d/*.sh]
        S2[routable.d/*.sh]
        S3[routes.d/*.sh]
        S4[*.d/*.sh]
    end

    NM --> BL
    SN --> BL
    DC --> BL

    BL --> ML
    AW --> ML
    RW --> ML
    LW --> ML

    ML --> NS
    NS --> RL

    RL --> RT
    ML --> SE
    ML --> BUS

    SE --> S1
    SE --> S2
    SE --> S3
    SE --> S4

    style ML fill:#e1f5ff
    style NS fill:#fff3e0
    style SE fill:#f3e5f5
    style RT fill:#e8f5e9
```

## 🔄 Event Flow

### systemd-networkd Backend

```mermaid
sequenceDiagram
    participant NW as systemd-networkd
    participant DB as DBus
    participant NE as netevd
    participant NL as Netlink
    participant ST as State Manager
    participant SC as Script Executor
    participant US as User Scripts

    Note over NW: Interface becomes<br/>routable

    NW->>DB: PropertiesChanged<br/>/org/freedesktop/network1/link/_33
    DB->>NE: DBus Signal

    par Parallel Processing
        NE->>NL: Query interface details
        NL-->>NE: IP addresses, routes

        NE->>NW: Read /run/systemd/netif/links/3
        NW-->>NE: State file data
    end

    NE->>NE: Build JSON representation
    NE->>ST: Update state (Arc::write)

    alt Routing rules configured
        NE->>NL: Add routing policy rules
        NE->>NL: Add custom route table
        NL-->>NE: Rules installed
    end

    NE->>SC: Execute routable.d scripts
    SC->>SC: Validate env variables
    SC->>US: Run scripts with ENV
    US-->>SC: Exit codes
    SC-->>NE: Results

    Note over NE: Ready for next event
```

### Real-time Netlink Events

```mermaid
sequenceDiagram
    participant K as Linux Kernel
    participant NL as Netlink Socket
    participant AW as Address Watcher
    participant RW as Route Watcher
    participant LW as Link Watcher
    participant ST as NetworkState
    participant RT as Route Config

    Note over K: Network change occurs

    par Concurrent Watchers
        K->>NL: NewAddress Event
        NL->>AW: Address message
        AW->>AW: Filter interface
        AW->>ST: Read state
        AW->>AW: Detect change
        AW->>RT: Configure routing
        RT->>K: Add policy rules
        AW->>ST: Update state

        K->>NL: NewRoute Event
        NL->>RW: Route message
        RW->>RW: Extract interface
        RW->>ST: Get link name
        RW->>RW: Execute scripts

        K->>NL: NewLink Event
        NL->>LW: Link message
        LW->>ST: Refresh links
        LW->>LW: Log change
    end

    Note over AW,LW: <100ms latency
```

## 🛣️ Routing Policy Rules Flow

```mermaid
graph LR
    subgraph "Interface: eth1 (index 3)"
        A[IP: 192.168.1.100<br/>Gateway: 192.168.1.1]
    end

    subgraph "netevd Logic"
        B[Detect routable state]
        C[Calculate table ID<br/>200 + 3 = 203]
        D[Discover gateway<br/>192.168.1.1]
    end

    subgraph "Kernel Configuration"
        E[Add rule: from 192.168.1.100<br/>lookup table 203]
        F[Add rule: to 192.168.1.100<br/>lookup table 203]
        G[Add route: default via 192.168.1.1<br/>dev eth1 table 203]
    end

    subgraph "Traffic Flow"
        H[Packet from 192.168.1.100]
        I[Rule match]
        J[Lookup table 203]
        K[Route via eth1]
    end

    A --> B
    B --> C
    C --> D
    D --> E
    D --> F
    D --> G

    E --> I
    F --> I

    H --> I
    I --> J
    G --> J
    J --> K

    style C fill:#ffeb3b
    style E fill:#4caf50
    style F fill:#4caf50
    style G fill:#4caf50
    style K fill:#2196f3
```

## 🔐 Security Model

```mermaid
sequenceDiagram
    participant SU as Start (root)
    participant PC as prctl syscall
    participant US as setuid/setgid
    participant CA as Capabilities
    participant NE as netevd process
    participant SC as Scripts

    SU->>SU: UID = 0 (root)

    Note over SU,PC: Step 1: Enable capability retention
    SU->>PC: PR_SET_KEEPCAPS = 1
    PC-->>SU: Capabilities will survive setuid

    Note over US: Step 2: Drop privileges
    SU->>US: setgid(netevd)
    SU->>US: setuid(netevd)
    US-->>NE: UID = netevd (non-root)

    Note over PC: Step 3: Disable capability retention
    NE->>PC: PR_SET_KEEPCAPS = 0

    Note over CA: Step 4: Apply minimal capabilities
    NE->>CA: Clear all capabilities
    NE->>CA: Set CAP_NET_ADMIN (permitted)
    NE->>CA: Set CAP_NET_ADMIN (effective)
    CA-->>NE: Network operations only

    Note over NE,SC: Step 5: Execute scripts as netevd
    NE->>SC: fork + exec (UID=netevd)
    SC-->>SC: No capabilities inherited
    SC-->>NE: Results

    Note over NE: Running as: netevd<br/>Capabilities: CAP_NET_ADMIN<br/>No root access
```

## 📡 Component Interaction

```mermaid
graph TB
    subgraph "Configuration Layer"
        CFG[config/mod.rs<br/>YAML Parser]
    end

    subgraph "Security Layer"
        USR[system/user.rs<br/>Privilege Drop]
        CAP[system/capability.rs<br/>CAP_NET_ADMIN]
        VAL[system/validation.rs<br/>Input Sanitization]
    end

    subgraph "Network Layer"
        LNK[network/link.rs<br/>Link Management]
        ADR[network/address.rs<br/>IP Addresses]
        RTE[network/route.rs<br/>Route Operations]
        RUL[network/routing_rule.rs<br/>Policy Rules]
        STA[network/mod.rs<br/>NetworkState]
    end

    subgraph "Event Listeners"
        NWD[listeners/networkd<br/>DBus Listener]
        NMR[listeners/networkmanager<br/>DBus Listener]
        DHC[listeners/dhclient<br/>File Watcher]
    end

    subgraph "System Integration"
        RES[bus/resolved.rs<br/>DNS Management]
        HST[bus/hostnamed.rs<br/>Hostname Management]
        EXE[system/execute.rs<br/>Script Execution]
    end

    CFG --> USR
    USR --> CAP

    NWD --> STA
    NMR --> STA
    DHC --> STA

    STA --> LNK
    STA --> ADR
    STA --> RTE
    STA --> RUL

    RTE --> VAL
    RUL --> VAL

    NWD --> RES
    NWD --> HST
    DHC --> RES
    DHC --> HST

    NWD --> EXE
    NMR --> EXE
    DHC --> EXE

    EXE --> VAL

    style STA fill:#ffeb3b
    style VAL fill:#f44336,color:#fff
    style CAP fill:#f44336,color:#fff
    style EXE fill:#4caf50
```

## 🔀 Network State Machine

```mermaid
stateDiagram-v2
    [*] --> NoCarrier: Interface added
    NoCarrier --> Carrier: Cable connected
    Carrier --> NoCarrier: Cable disconnected

    Carrier --> Configured: DHCP/Static IP
    Configured --> Degraded: Partial config
    Degraded --> Configured: Config fixed

    Configured --> Routable: Gateway reachable
    Routable --> Configured: Gateway lost

    Routable --> [*]: Interface removed
    NoCarrier --> [*]: Interface removed

    note right of NoCarrier
        Scripts: no-carrier.d/
        No IP address
    end note

    note right of Carrier
        Scripts: carrier.d/
        Physical link up
    end note

    note right of Configured
        Scripts: configured.d/
        IP assigned
    end note

    note right of Degraded
        Scripts: degraded.d/
        Issues detected
    end note

    note right of Routable
        Scripts: routable.d/
        Full connectivity
        Routing rules applied
    end note
```

## 📦 Deployment Architecture

```mermaid
graph TB
    subgraph "System Boot"
        SYS[systemd]
    end

    subgraph "netevd Service"
        BIN["Binary: /usr/bin/netevd"]
        CFG["Config: /etc/netevd/netevd.yaml"]
        USR["User: netevd"]
        CAP["Capabilities: CAP_NET_ADMIN"]
    end

    subgraph "User Scripts"
        SC1["carrier.d/"]
        SC2["routable.d/"]
        SC3["routes.d/"]
        SC4["activated.d/"]
    end

    subgraph "System Services"
        NWD[systemd-networkd]
        RES[systemd-resolved]
        HST[systemd-hostnamed]
        NMG[NetworkManager]
    end

    subgraph "Kernel"
        NET[Netlink Socket]
        RTB[Routing Tables]
        RUL[Policy Rules]
    end

    SYS -->|Starts| BIN
    BIN -->|Reads| CFG
    BIN -->|Runs as| USR
    BIN -->|Requires| CAP

    BIN <-->|DBus| NWD
    BIN <-->|DBus| NMG
    BIN <-->|DBus| RES
    BIN <-->|DBus| HST

    BIN <-->|Subscribe| NET
    BIN -->|Configure| RTB
    BIN -->|Configure| RUL

    BIN -->|Execute| SC1
    BIN -->|Execute| SC2
    BIN -->|Execute| SC3
    BIN -->|Execute| SC4

    style BIN fill:#2196f3,color:#fff
    style USR fill:#4caf50,color:#fff
    style CAP fill:#ff9800,color:#fff
    style NET fill:#9c27b0,color:#fff
```

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Use Cases & Examples](#use-cases--examples)
- [Configuration](#configuration)
- [Advanced Usage](#advanced-usage)
- [Building from Source](#building-from-source)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/ssahani/netevd.git
cd netevd

# Build and install
cargo build --release
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service
sudo install -Dm644 examples/netevd.yaml /etc/netevd/netevd.yaml

# Create script directories
sudo mkdir -p /etc/netevd/{carrier.d,configured.d,degraded.d,manager.d,no-carrier.d,routable.d,routes.d,activated.d,disconnected.d}

# Create netevd user
sudo useradd -M -s /usr/bin/nologin netevd

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
```

### Basic Configuration

Edit `/etc/netevd/netevd.yaml`:

```yaml
system:
  log_level: "info"
  backend: "systemd-networkd"  # or "NetworkManager" or "dhclient"

network:
  links: "eth0 eth1"  # Interfaces to monitor
  routing_policy_rules: "eth1"  # Interfaces needing custom routing
  emit_json: true
  use_dns: false
  use_domain: false
  use_hostname: false
```

## Use Cases & Examples

### Example 1: Run a Command When DHCP Address is Acquired

#### With systemd-networkd

Create an executable script `/etc/netevd/routable.d/01-notify.sh`:

```bash
#!/bin/bash
# This script runs when an interface becomes routable

# Available environment variables:
# - LINK: Interface name (e.g., "eth0")
# - LINKINDEX: Interface index number
# - STATE: Current state ("routable")
# - BACKEND: Event source ("systemd-networkd")
# - ADDRESSES: Space-separated list of IP addresses
# - JSON: Full interface information in JSON format

echo "Interface $LINK ($LINKINDEX) is now routable"
echo "IP Addresses: $ADDRESSES"

# Example: Send notification
notify-send "Network Ready" "Interface $LINK is now routable with IPs: $ADDRESSES"

# Example: Start a service that depends on network
systemctl start myapp.service

# Example: Update DNS configuration
echo "nameserver $DNS" >> /etc/resolv.conf.d/custom
```

Make it executable:
```bash
sudo chmod +x /etc/netevd/routable.d/01-notify.sh
```

#### With NetworkManager

Create `/etc/netevd/activated.d/01-network-ready.sh`:

```bash
#!/bin/bash
# Runs when NetworkManager activates an interface

logger -t netevd "Interface $LINK activated with state: $STATE"

if [ "$STATE" = "activated" ]; then
    # Your custom logic here
    /usr/local/bin/update-vpn-routes.sh "$LINK"
fi
```

#### With dhclient

Configure dhclient mode in `/etc/netevd/netevd.yaml`:

```yaml
system:
  backend: "dhclient"

network:
  use_dns: true
  use_domain: true
  use_hostname: true
```

Create `/etc/netevd/routable.d/01-dhcp-lease.sh`:

```bash
#!/bin/bash
# Available DHCP lease variables:
# - DHCP_ADDRESS: Assigned IP address
# - DHCP_GATEWAY: Default gateway
# - DHCP_DNS: DNS servers
# - DHCP_DOMAIN: Domain name
# - DHCP_HOSTNAME: Hostname from DHCP

echo "Got DHCP lease for $LINK: $DHCP_ADDRESS"
echo "Gateway: $DHCP_GATEWAY"
echo "DNS: $DHCP_DNS"
```

### Example 2: Multi-Interface Routing (Secondary Network Interface)

**Problem**: You have two interfaces (eth0 and eth1) in the same subnet. Traffic arriving via eth1 tries to leave via eth0 (default gateway), breaking return packets.

**Solution**: Use routing policy rules to ensure traffic arriving on eth1 leaves via eth1.

#### Configuration

Edit `/etc/netevd/netevd.yaml`:

```yaml
system:
  backend: "systemd-networkd"

network:
  routing_policy_rules: "eth1"  # Configure routing for eth1
```

#### What happens automatically:

1. When eth1 gets an IP address (e.g., 192.168.1.100)
2. `netevd` creates a custom routing table (table ID = 200 + ifindex)
3. Adds routing policy rules:
   - `from 192.168.1.100 lookup 203` (assuming ifindex=3)
   - `to 192.168.1.100 lookup 203`
4. Adds default route in table 203 via eth1's gateway

#### Verify it works:

```bash
# View routing policy rules
ip rule list

# You should see:
# 32765:  from 192.168.1.100 lookup 203
# 32766:  to 192.168.1.100 lookup 203

# View custom routing table
ip route show table 203

# You should see:
# default via 192.168.1.1 dev eth1
```

#### Test connectivity:

```bash
# Send traffic from eth1's IP
curl --interface eth1 https://example.com

# Verify with tcpdump
sudo tcpdump -i eth1 -n host 192.168.1.100
```

### Example 3: Execute Custom Scripts on Link State Changes

#### Monitor carrier loss and notify

Create `/etc/netevd/no-carrier.d/01-alert.sh`:

```bash
#!/bin/bash
# Runs when interface loses carrier (cable unplugged)

ALERT_EMAIL="admin@example.com"

echo "Interface $LINK lost carrier at $(date)" | \
    mail -s "Network Alert: Link Down on $(hostname)" "$ALERT_EMAIL"

# Log to syslog
logger -t netevd -p daemon.warning "Link $LINK carrier lost"

# Could also: disable services, trigger failover, etc.
```

#### Auto-reconnect WiFi

Create `/etc/netevd/disconnected.d/01-wifi-reconnect.sh`:

```bash
#!/bin/bash
# Auto-reconnect WiFi when NetworkManager disconnects

if [ "$BACKEND" = "NetworkManager" ] && [ "$STATE" = "disconnected" ]; then
    # Wait a bit
    sleep 5

    # Try to reconnect
    nmcli device connect "$LINK"

    logger -t netevd "Attempted to reconnect $LINK"
fi
```

### Example 4: Dynamic DNS Updates

Create `/etc/netevd/routable.d/02-update-dns.sh`:

```bash
#!/bin/bash
# Update dynamic DNS when IP changes

DDNS_HOSTNAME="myhost.dyndns.org"
DDNS_TOKEN="your-api-token"

# Extract first IPv4 address
IP=$(echo "$ADDRESSES" | awk '{print $1}')

if [[ "$IP" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    # Update dynamic DNS (example for Cloudflare)
    curl -X PUT "https://api.cloudflare.com/client/v4/zones/ZONE_ID/dns_records/RECORD_ID" \
         -H "Authorization: Bearer $DDNS_TOKEN" \
         -H "Content-Type: application/json" \
         --data "{\"type\":\"A\",\"name\":\"$DDNS_HOSTNAME\",\"content\":\"$IP\"}"

    logger -t netevd "Updated DDNS for $LINK: $IP"
fi
```

### Example 5: JSON Processing with jq

systemd-networkd provides rich JSON data. Create `/etc/netevd/routable.d/03-process-json.sh`:

```bash
#!/bin/bash
# Process JSON data from systemd-networkd

if [ -z "$JSON" ]; then
    echo "No JSON data available"
    exit 0
fi

# Parse JSON with jq
MTU=$(echo "$JSON" | jq -r '.MTU')
DRIVER=$(echo "$JSON" | jq -r '.Driver')
IPV4_STATE=$(echo "$JSON" | jq -r '.IPv4AddressState')
DNS_SERVERS=$(echo "$JSON" | jq -r '.DNS[]' | tr '\n' ' ')

echo "Interface: $LINK"
echo "  MTU: $MTU"
echo "  Driver: $DRIVER"
echo "  IPv4 State: $IPV4_STATE"
echo "  DNS Servers: $DNS_SERVERS"

# Example: Adjust MTU if needed
if [ "$MTU" -gt 1500 ]; then
    ip link set dev "$LINK" mtu 1500
    logger -t netevd "Adjusted MTU on $LINK to 1500"
fi

# Example: Log all addresses
echo "$JSON" | jq -r '.Address[] | "  \(.IP)/\(.Mask)"'
```

### Example 6: VPN Integration

Create `/etc/netevd/routable.d/04-vpn-routes.sh`:

```bash
#!/bin/bash
# Add custom routes when VPN interface comes up

VPN_INTERFACE="wg0"
OFFICE_NETWORK="10.0.0.0/8"
REMOTE_NETWORK="172.16.0.0/12"

if [ "$LINK" = "$VPN_INTERFACE" ] && [ "$STATE" = "routable" ]; then
    # Add routes to private networks via VPN
    ip route add $OFFICE_NETWORK dev $VPN_INTERFACE
    ip route add $REMOTE_NETWORK dev $VPN_INTERFACE

    logger -t netevd "Added VPN routes for $LINK"

    # Update firewall rules
    iptables -A FORWARD -i $VPN_INTERFACE -j ACCEPT
    iptables -A FORWARD -o $VPN_INTERFACE -j ACCEPT
fi
```

## Directory Structure

`netevd` uses the following directories in `/etc/netevd/`:

```
/etc/netevd/
├── netevd.yaml              # Main configuration file
├── carrier.d/               # Link has carrier (cable connected)
├── no-carrier.d/            # Link lost carrier (cable disconnected)
├── configured.d/            # Link is configured (systemd-networkd)
├── degraded.d/              # Link is degraded (systemd-networkd)
├── routable.d/              # Link is routable (has working network)
├── activated.d/             # Device activated (NetworkManager)
├── disconnected.d/          # Device disconnected (NetworkManager)
├── manager.d/               # Network manager state changes
└── routes.d/                # Route changes detected
```

**Script Execution Rules:**
- Scripts must be executable (`chmod +x`)
- Scripts are executed in alphabetical order (prefix with numbers: `01-`, `02-`, etc.)
- Scripts receive environment variables with network state information
- Non-zero exit codes are logged but don't stop other scripts

## Configuration

### Configuration File: `/etc/netevd/netevd.yaml`

#### System Section

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `log_level` | `trace`, `debug`, `info`, `warn`, `error` | `info` | Logging verbosity |
| `backend` | `systemd-networkd`, `NetworkManager`, `dhclient` | `systemd-networkd` | Network event source |

#### Network Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `links` | String | (all) | Space-separated list of interfaces to monitor |
| `routing_policy_rules` | String | (none) | Interfaces needing custom routing tables |
| `emit_json` | Boolean | `true` | Emit JSON data (systemd-networkd only) |
| `use_dns` | Boolean | `false` | Send DNS to systemd-resolved (dhclient only) |
| `use_domain` | Boolean | `false` | Send domain to systemd-resolved (dhclient only) |
| `use_hostname` | Boolean | `false` | Send hostname to systemd-hostnamed (dhclient only) |

### Complete Configuration Examples

#### Example 1: Laptop with WiFi

```yaml
system:
  log_level: "info"
  backend: "NetworkManager"

network:
  links: "wlan0"
  routing_policy_rules: ""
  emit_json: false
```

#### Example 2: Server with Multiple NICs

```yaml
system:
  log_level: "warn"
  backend: "systemd-networkd"

network:
  links: "eth0 eth1 eth2"
  routing_policy_rules: "eth1 eth2"  # eth0 is primary
  emit_json: true
```

#### Example 3: Legacy System with dhclient

```yaml
system:
  log_level: "debug"
  backend: "dhclient"

network:
  links: "eth0"
  use_dns: true
  use_domain: true
  use_hostname: true
```

## Advanced Usage

### Environment Variables Available to Scripts

#### Common Variables (All Generators)

- `LINK`: Interface name (e.g., `eth0`)
- `LINKINDEX`: Interface index number
- `STATE`: Current state (e.g., `routable`, `activated`, `disconnected`)
- `BACKEND`: Source of event (`systemd-networkd`, `NetworkManager`, `dhclient`)
- `ADDRESSES`: Space-separated list of IP addresses on the interface

#### systemd-networkd Specific

- `JSON`: Full interface data in JSON format (if `emit_json: true`)
  - Includes: MTU, MAC address, driver, operational state, DNS, routes, etc.

#### dhclient Specific

- `DHCP_ADDRESS`: IP address from DHCP lease
- `DHCP_GATEWAY`: Default gateway
- `DHCP_DNS`: DNS servers
- `DHCP_DOMAIN`: Domain name
- `DHCP_HOSTNAME`: Hostname from DHCP
- `DHCP_LEASE`: Full lease information

#### NetworkManager Specific

All common variables plus NetworkManager device state information.

### Custom Routing Tables

When `routing_policy_rules` is configured for an interface, `netevd` automatically:

1. **Calculates table ID**: `200 + interface_index`
   - eth0 (index 2) → table 202
   - eth1 (index 3) → table 203
   - wlan0 (index 4) → table 204

2. **Creates routing policy rules**:
   ```bash
   ip rule add from <interface_ip> table <table_id>
   ip rule add to <interface_ip> table <table_id>
   ```

3. **Adds default route**:
   ```bash
   ip route add default via <gateway> dev <interface> table <table_id>
   ```

4. **Cleanup**: When address is removed, rules and routes are automatically deleted

### Monitoring netevd

```bash
# Check service status
sudo systemctl status netevd

# View logs
sudo journalctl -u netevd -f

# View recent logs with context
sudo journalctl -u netevd -n 100

# Filter by priority
sudo journalctl -u netevd -p warning

# Check which scripts are being executed
sudo journalctl -u netevd | grep "Executing"
```

### Testing Scripts Manually

```bash
# Set environment variables and run script
sudo env LINK=eth0 LINKINDEX=2 STATE=routable BACKEND=systemd-networkd \
     ADDRESSES="192.168.1.100" \
     /etc/netevd/routable.d/01-test.sh

# Test with JSON data
sudo env LINK=eth0 JSON='{"Index":2,"Name":"eth0","OperState":"up"}' \
     /etc/netevd/routable.d/02-json-test.sh
```

## Building from Source

### Prerequisites

- Rust 1.70 or later
- Cargo
- Linux with systemd (for full functionality)

### Build

```bash
# Clone repository
git clone https://github.com/ssahani/netevd.git
cd netevd

# Build in release mode
cargo build --release

# Run tests
cargo test

# Check for issues
cargo clippy
```

### Installation

```bash
# Install binary
sudo install -Dm755 target/release/netevd /usr/bin/netevd

# Install systemd service
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service

# Install configuration
sudo install -Dm644 examples/netevd.yaml /etc/netevd/netevd.yaml

# Create user
sudo useradd -r -s /usr/bin/nologin -d /nonexistent netevd

# Create script directories
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}

# Enable service
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
```

## Troubleshooting

### Service won't start

```bash
# Check service status
sudo systemctl status netevd

# View full logs
sudo journalctl -u netevd -n 100 --no-pager

# Common issues:
# 1. User doesn't exist
sudo useradd -r -s /usr/bin/nologin netevd

# 2. Configuration file syntax error
netevd --config /etc/netevd/netevd.yaml

# 3. Permission issues
sudo chown -R netevd:netevd /etc/netevd/
```

### Scripts not executing

```bash
# Check if scripts are executable
ls -la /etc/netevd/routable.d/

# Make scripts executable
sudo chmod +x /etc/netevd/routable.d/*.sh

# Check logs for script execution
sudo journalctl -u netevd | grep "Executing"

# Test script manually
sudo bash -x /etc/netevd/routable.d/01-test.sh
```

### Routing policy rules not working

```bash
# Check if interface is configured for routing rules
grep routing_policy_rules /etc/netevd/netevd.yaml

# View current rules
ip rule list

# View custom routing tables
ip route show table 202  # Adjust table number

# Check netevd logs
sudo journalctl -u netevd | grep "routing"
```

### No events received

```bash
# For systemd-networkd:
# Check if networkd is running
systemctl status systemd-networkd

# Trigger an event
sudo networkctl reload

# For NetworkManager:
systemctl status NetworkManager
nmcli device status

# For dhclient:
# Check if dhclient is running
ps aux | grep dhclient

# Restart dhclient to generate events
sudo systemctl restart dhclient
```

### Debug logging

```bash
# Enable debug logging
sudo sed -i 's/log_level: "info"/log_level: "debug"/' /etc/netevd/netevd.yaml
sudo systemctl restart netevd

# Or set via environment variable
sudo systemctl edit netevd

# Add:
[Service]
Environment="RUST_LOG=debug"

sudo systemctl daemon-reload
sudo systemctl restart netevd
```

## 🏗️ Technical Architecture

### Technology Stack

```mermaid
graph LR
    subgraph "Core Runtime"
        TOK[Tokio 1.35<br/>Async Runtime]
    end

    subgraph "Network Communication"
        RTN[rtnetlink 0.14<br/>Netlink Operations]
        NPR[netlink-packet-route<br/>Protocol Messages]
    end

    subgraph "DBus Integration"
        ZBS[zbus 4.0<br/>Async DBus]
    end

    subgraph "File System"
        NOT[notify 6.1<br/>FS Events]
    end

    subgraph "Security"
        NIX[nix 0.29<br/>Unix APIs]
        CPS[caps 0.5<br/>Capabilities]
    end

    subgraph "Configuration"
        SER[serde + serde_yaml<br/>Parsing]
        CFP[configparser<br/>INI Files]
    end

    TOK --> RTN
    TOK --> ZBS
    TOK --> NOT

    RTN --> NPR

    style TOK fill:#61dafb,color:#000
    style RTN fill:#4caf50,color:#fff
    style ZBS fill:#ff9800,color:#fff
    style NIX fill:#f44336,color:#fff
```

### Module Architecture

```mermaid
graph TB
    subgraph "src/"
        MAIN[main.rs<br/>Entry Point<br/>Privilege Drop<br/>Event Loop]

        subgraph "config/"
            CFG[mod.rs<br/>YAML Parsing<br/>Validation]
        end

        subgraph "network/"
            NET_M[mod.rs<br/>NetworkState]
            NET_L[link.rs<br/>Link Management]
            NET_A[address.rs<br/>IP Operations]
            NET_R[route.rs<br/>Route Ops]
            NET_RR[routing_rule.rs<br/>Policy Rules]
            NET_W[watcher.rs<br/>Netlink Events]
        end

        subgraph "listeners/"
            LIS_N[networkd/<br/>DBus + State Files]
            LIS_NM[networkmanager/<br/>DBus Signals]
            LIS_D[dhclient/<br/>Lease Parser]
        end

        subgraph "bus/"
            BUS_R[resolved.rs<br/>DNS via DBus]
            BUS_H[hostnamed.rs<br/>Hostname via DBus]
        end

        subgraph "system/"
            SYS_C[capability.rs<br/>CAP_NET_ADMIN]
            SYS_U[user.rs<br/>setuid/setgid]
            SYS_E[execute.rs<br/>Script Exec]
            SYS_V[validation.rs<br/>Input Filter]
            SYS_P[paths.rs<br/>File Utils]
        end
    end

    MAIN --> CFG
    MAIN --> SYS_U
    MAIN --> SYS_C

    MAIN --> NET_W
    MAIN --> LIS_N
    MAIN --> LIS_NM
    MAIN --> LIS_D

    NET_W --> NET_M
    NET_W --> NET_L
    NET_W --> NET_A
    NET_W --> NET_R
    NET_W --> NET_RR

    LIS_N --> NET_M
    LIS_N --> BUS_R
    LIS_N --> BUS_H
    LIS_N --> SYS_E

    LIS_NM --> NET_M
    LIS_NM --> SYS_E

    LIS_D --> BUS_R
    LIS_D --> BUS_H
    LIS_D --> SYS_E

    SYS_E --> SYS_V
    SYS_E --> SYS_P

    NET_R --> SYS_V
    NET_RR --> SYS_V

    style MAIN fill:#2196f3,color:#fff
    style NET_M fill:#ffeb3b
    style SYS_V fill:#f44336,color:#fff
    style SYS_E fill:#4caf50
```

### Concurrency Model

All watchers and listeners run concurrently using `tokio::select!`:

```rust
tokio::select! {
    _ = watch_addresses() => {},      // Netlink address events
    _ = watch_routes() => {},         // Netlink route events
    _ = watch_links() => {},          // Netlink link events
    _ = spawn_listener() => {},       // DBus or file watcher
    _ = signal_handler() => {},       // SIGTERM/SIGINT
}
```

### Data Flow Pipeline

```mermaid
graph LR
    subgraph "Input"
        I1[Netlink Events]
        I2[DBus Signals]
        I3[File Changes]
    end

    subgraph "Processing"
        P1[Event Filtering]
        P2[State Update<br/>Arc RwLock]
        P3[Validation]
    end

    subgraph "Actions"
        A1[Route Config]
        A2[Script Exec]
        A3[DBus Calls]
    end

    subgraph "Output"
        O1[Routing Tables]
        O2[User Scripts]
        O3[System Services]
    end

    I1 --> P1
    I2 --> P1
    I3 --> P1

    P1 --> P2
    P2 --> P3

    P3 --> A1
    P3 --> A2
    P3 --> A3

    A1 --> O1
    A2 --> O2
    A3 --> O3

    style P2 fill:#ffeb3b
    style P3 fill:#f44336,color:#fff
```

## 🔒 Security

`netevd` implements defense-in-depth security with multiple layers:

### Security Layers

```mermaid
graph TD
    subgraph "Layer 1: Privilege Separation"
        L1A[Start as root UID=0]
        L1B[Drop to netevd user]
        L1C[Cannot regain root]
    end

    subgraph "Layer 2: Minimal Capabilities"
        L2A[Clear all capabilities]
        L2B[Set CAP_NET_ADMIN only]
        L2C[No capability inheritance]
    end

    subgraph "Layer 3: Input Validation"
        L3A[Validate interface names]
        L3B[Sanitize IP addresses]
        L3C[Filter shell metacharacters]
        L3D[Reject command injection]
    end

    subgraph "Layer 4: Execution Isolation"
        L4A[Scripts run as netevd]
        L4B[No capabilities passed]
        L4C[Validated environment only]
    end

    subgraph "Layer 5: System Hardening"
        L5A[NoNewPrivileges=true]
        L5B[ProtectSystem=strict]
        L5C[PrivateTmp=true]
    end

    L1A --> L1B --> L1C
    L2A --> L2B --> L2C
    L3A --> L3B --> L3C --> L3D
    L4A --> L4B --> L4C
    L5A --> L5B --> L5C

    L1C --> L2A
    L2C --> L3A
    L3D --> L4A
    L4C --> L5A

    style L1B fill:#4caf50,color:#fff
    style L2B fill:#ff9800,color:#fff
    style L3D fill:#f44336,color:#fff
    style L4B fill:#2196f3,color:#fff
```

### Security Features

1. **Privilege Dropping**: Starts as root, drops to `netevd` user
2. **Minimal Capabilities**: Retains only `CAP_NET_ADMIN` (network configuration)
3. **Capability Retention**: Uses `prctl(PR_SET_KEEPCAPS)` for safe privilege drop
4. **Input Validation**: All environment variables validated before script execution
5. **Script Execution**: Scripts run as `netevd` user with no capabilities
6. **No Shell Injection**: Dangerous characters rejected (`;`, `$`, backticks, etc.)
7. **Systemd Hardening**: NoNewPrivileges, ProtectSystem, PrivateTmp

### Threat Model & Mitigations

| Threat | Mitigation |
|--------|------------|
| **Malicious DHCP Server** | Input validation rejects shell metacharacters |
| **Command Injection** | Environment variables sanitized, dangerous patterns blocked |
| **Privilege Escalation** | Runs as `netevd` user, NoNewPrivileges prevents setuid |
| **Capability Leakage** | Scripts inherit no capabilities |
| **System File Tampering** | ProtectSystem=strict, read-only filesystem |
| **Resource Exhaustion** | Systemd resource limits (optional) |

### Capability Details

```bash
# View capabilities (if using systemd)
sudo systemctl show netevd | grep Capabilit

# Should show:
# AmbientCapabilities=cap_net_admin
# CapabilityBoundingSet=cap_net_admin

# Binary capabilities (alternative to systemd)
sudo getcap /usr/bin/netevd
# /usr/bin/netevd = cap_net_admin+eip
```

### Validation Examples

```rust
// Interface names: only alphanumeric, _, -, .
validate_interface_name("eth0")       // ✅ Pass
validate_interface_name("eth0; rm")   // ❌ Reject

// IP addresses: strict parsing
validate_ip_address("192.168.1.1")    // ✅ Pass
validate_ip_address("192.168.1.1; whoami") // ❌ Reject

// Hostnames: RFC compliant
validate_hostname("example.com")      // ✅ Pass
validate_hostname("$(whoami).com")    // ❌ Reject

// Environment values: no shell metacharacters
sanitize_env_value("safe-value")      // ✅ Pass
sanitize_env_value("value && malicious") // ❌ Reject
```

## ⚡ Performance

### Benchmarks

Performance metrics on modern hardware (4-core CPU, 8GB RAM):

```mermaid
graph LR
    subgraph "Resource Usage"
        M[Memory: 3-5 MB RSS]
        C[CPU: <1% idle<br/>2-5% during events]
    end

    subgraph "Latency"
        S[Startup: <100ms]
        E[Event Processing:<br/><100ms]
        D[DBus→Script:<br/><10ms]
    end

    subgraph "Throughput"
        EV[Events: >1000/sec]
        SC[Scripts: Limited by exec]
    end

    style M fill:#4caf50,color:#fff
    style C fill:#4caf50,color:#fff
    style E fill:#2196f3,color:#fff
    style D fill:#2196f3,color:#fff
```

### Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| **Memory (Idle)** | 3-5 MB RSS | Minimal footprint |
| **Memory (Active)** | 5-8 MB RSS | During event processing |
| **CPU (Idle)** | <1% | Async I/O, event-driven |
| **CPU (Events)** | 2-5% | Brief spikes during processing |
| **Startup Time** | <100ms | Fast boot integration |
| **Event Latency** | <100ms | Netlink multicast subscription |
| **Script Latency** | <10ms | From event to script execution |
| **Concurrent Events** | 1000+/sec | Async processing with tokio |

### Comparison vs Polling

```mermaid
gantt
    title Event Latency Comparison
    dateFormat X
    axisFormat %Ls

    section Netlink Events
    Event occurs: milestone, 0, 0
    netevd detects: milestone, 50, 50
    Script executes: milestone, 60, 60

    section Polling (5s)
    Event occurs: milestone, 0, 0
    Poll interval: 0, 5000
    netevd detects: milestone, 5000, 5000
    Script executes: milestone, 5010, 5010
```

**Result**: Real-time events are **50-100x faster** than 5-second polling

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo clippy`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Development Setup

```bash
# Install development tools
rustup component add rustfmt clippy

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build documentation
cargo doc --open
```

## License

[LGPL-3.0-or-later](https://www.gnu.org/licenses/lgpl-3.0.html)

Copyright 2026 Susant Sahani

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

## Author

**Susant Sahani** <<ssahani@redhat.com>>

## Links

- [GitHub Repository](https://github.com/ssahani/netevd)
- [Issue Tracker](https://github.com/ssahani/netevd/issues)
- [systemd-networkd Documentation](https://www.freedesktop.org/software/systemd/man/systemd-networkd.html)
- [NetworkManager Documentation](https://networkmanager.dev/)
