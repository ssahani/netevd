# netevd

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![CI](https://github.com/ssahani/netevd/actions/workflows/ci.yml/badge.svg)](https://github.com/ssahani/netevd/actions/workflows/ci.yml)
[![Functional Tests](https://github.com/ssahani/netevd/actions/workflows/functional-tests.yml/badge.svg)](https://github.com/ssahani/netevd/actions/workflows/functional-tests.yml)
[![codecov](https://codecov.io/gh/ssahani/netevd/branch/main/graph/badge.svg)](https://codecov.io/gh/ssahani/netevd)

**netevd** is a network event daemon that watches your Linux network interfaces and runs scripts when things change. Think of it as systemd path units, but purpose-built for networking: when an interface gets an IP, loses its link, or routes change, netevd executes your scripts with full context about what happened.

It bridges **systemd-networkd**, **NetworkManager**, and **dhclient** into a single, unified event system -- with automatic policy routing, a REST API, Prometheus metrics, and a defense-in-depth security model.

## Why netevd?

| Problem | netevd solution |
|---------|----------------|
| Need scripts to run when network state changes | Drop scripts in `/etc/netevd/routable.d/` -- done |
| Multi-homed server with broken return-path routing | Automatic per-interface routing tables and policy rules |
| Want real-time network events, not polling | Netlink multicast: sub-100ms latency, zero polling |
| Need to support multiple network managers | One daemon handles networkd, NetworkManager, and dhclient |
| Security concerns with network daemons | Privilege separation, CAP_NET_ADMIN only, input validation |

## Quick Start

```bash
# Build and install
git clone https://github.com/ssahani/netevd.git && cd netevd
cargo build --release
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service
sudo install -Dm644 examples/netevd.yaml /etc/netevd/netevd.yaml

# Set up
sudo useradd -r -M -s /usr/bin/nologin netevd
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}

# Start
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
```

Create your first script -- this runs whenever an interface becomes fully routable:

```bash
cat <<'EOF' | sudo tee /etc/netevd/routable.d/01-notify.sh && sudo chmod +x /etc/netevd/routable.d/01-notify.sh
#!/bin/bash
logger -t netevd "Interface $LINK is routable: $ADDRESSES"
EOF
```

For the full walkthrough, see the **[Quick Start Guide](docs/QUICKSTART.md)**.

## How It Works

```
                    +------------------+
                    |   Linux Kernel   |
                    |  Netlink events  |
                    +--------+---------+
                             |
         +-------------------+-------------------+
         |                   |                   |
   +-----------+      +-----------+      +-----------+
   | Addresses |      |   Links   |      |  Routes   |
   |  watcher  |      |  watcher  |      |  watcher  |
   +-----+-----+      +-----+-----+      +-----+-----+
         |                   |                   |
         +-------------------+-------------------+
                             |
                    +--------+---------+
                    |  NetworkState    |
                    |  (Arc<RwLock>)   |
                    +--------+---------+
                             |
              +--------------+--------------+
              |              |              |
        +-----+-----+  +----+----+  +------+------+
        |  Routing   |  | Script  |  |    DBus     |
        |  policy    |  |  exec   |  |  resolved/  |
        |  rules     |  |         |  |  hostnamed  |
        +------------+  +---------+  +-------------+
```

**Event sources** -- netevd subscribes to kernel netlink multicast groups and listens for DBus signals from your chosen backend (systemd-networkd, NetworkManager) or watches dhclient lease files via inotify.

**State management** -- All state is held in a single `NetworkState` behind `Arc<RwLock>`, updated by concurrent Tokio tasks. Read locks for queries, write locks for mutations -- no races.

**Actions** -- On state changes, netevd configures routing policy rules, executes scripts from the matching event directory, and optionally pushes DNS/hostname updates via DBus.

## Configuration

```yaml
# /etc/netevd/netevd.yaml
system:
  log_level: "info"
  backend: "systemd-networkd"    # or "NetworkManager" or "dhclient"

monitoring:
  interfaces:                    # empty = monitor all
    - eth0
    - eth1

routing:
  policy_rules:                  # auto-create per-interface routing tables
    - eth1

backends:
  systemd_networkd:
    emit_json: true              # pass full JSON to scripts via $JSON
  dhclient:
    use_dns: false
    use_domain: false
    use_hostname: false
  networkmanager: {}
```

Full reference: **[Configuration Guide](CONFIGURATION.md)**

## Script Directories

Scripts are organized by the event that triggers them:

| Directory | Trigger | Backends |
|-----------|---------|----------|
| `carrier.d/` | Cable connected | All |
| `no-carrier.d/` | Cable disconnected | All |
| `configured.d/` | Interface has IP | systemd-networkd |
| `degraded.d/` | Partial configuration | systemd-networkd |
| `routable.d/` | Full connectivity | systemd-networkd, dhclient |
| `activated.d/` | Device activated | NetworkManager |
| `disconnected.d/` | Device disconnected | NetworkManager |
| `manager.d/` | Manager state change | All |
| `routes.d/` | Routing table change | All |

Scripts run in alphabetical order. Use numeric prefixes (`01-`, `02-`) to control ordering. Non-zero exit codes are logged but don't block other scripts.

### Environment Variables

Every script receives:

| Variable | Example |
|----------|---------|
| `$LINK` | `eth0` |
| `$LINKINDEX` | `2` |
| `$STATE` | `routable` |
| `$BACKEND` | `systemd-networkd` |
| `$ADDRESSES` | `192.168.1.100 10.0.0.5` |

**systemd-networkd** adds `$JSON` with full interface data (MTU, driver, DNS, routes).
**dhclient** adds `$DHCP_ADDRESS`, `$DHCP_GATEWAY`, `$DHCP_DNS`, `$DHCP_DOMAIN`, `$DHCP_HOSTNAME`.

## Automatic Policy Routing

For multi-homed servers, netevd solves the classic "wrong interface" problem automatically. When you list an interface under `routing.policy_rules`, netevd:

1. Creates a custom routing table (ID = 200 + interface index)
2. Adds `from <ip> lookup <table>` and `to <ip> lookup <table>` rules
3. Installs a default route via the interface's gateway in that table
4. Cleans up automatically when addresses are removed

```bash
# After netevd configures eth1 (index 3, IP 192.168.1.100):
$ ip rule list
32765: from 192.168.1.100 lookup 203
32766: to 192.168.1.100 lookup 203

$ ip route show table 203
default via 192.168.1.1 dev eth1
```

## Security

netevd follows a defense-in-depth model:

1. **Privilege separation** -- Starts as root, immediately drops to the `netevd` user via `setuid`/`setgid`
2. **Minimal capabilities** -- Retains only `CAP_NET_ADMIN`; child processes inherit nothing
3. **Input validation** -- All external data (interface names, IPs, hostnames) is validated; shell metacharacters are rejected
4. **No shell intermediary** -- Scripts are executed directly, not via `sh -c`
5. **systemd hardening** -- `NoNewPrivileges`, `ProtectSystem=strict`, `PrivateTmp`

Details: **[Security Policy](SECURITY.md)**

## Performance

| Metric | Value |
|--------|-------|
| Memory (idle) | 3-5 MB RSS |
| CPU (idle) | < 1% |
| Event latency | < 100ms (netlink multicast) |
| Event-to-script | < 10ms |
| Throughput | 1000+ events/sec |

## REST API

9 endpoints built on Axum for remote management and monitoring:

```bash
curl http://localhost:9090/api/v1/status       # Daemon status
curl http://localhost:9090/api/v1/interfaces    # List interfaces
curl http://localhost:9090/api/v1/routes        # Routing table
curl http://localhost:9090/api/v1/events        # Event history
curl http://localhost:9090/metrics              # Prometheus metrics
curl http://localhost:9090/health               # Health check
```

Full reference: **[API Documentation](docs/API.md)**

## Documentation

| Guide | Description |
|-------|-------------|
| **[Quick Start](docs/QUICKSTART.md)** | Up and running in 5 minutes |
| **[Installation](INSTALL.md)** | All platforms and package managers |
| **[Configuration](CONFIGURATION.md)** | Complete YAML reference |
| **[Examples](docs/EXAMPLES.md)** | Multi-homing, VPN, HA, DDNS, containers |
| **[REST API](docs/API.md)** | HTTP endpoints and data models |
| **[Metrics](docs/METRICS.md)** | Prometheus metrics and Grafana dashboards |
| **[Architecture](docs/ARCHITECTURE.md)** | Internals, concurrency, event pipeline |
| **[Troubleshooting](docs/TROUBLESHOOTING.md)** | Diagnosis and common fixes |
| **[Security](SECURITY.md)** | Threat model and hardening |
| **[Contributing](CONTRIBUTING.md)** | Dev setup and PR guidelines |
| **[Roadmap](ROADMAP.md)** | Planned features and priorities |
| **[Changelog](CHANGELOG.md)** | Release history |

## Contributing

```bash
git clone https://github.com/ssahani/netevd.git && cd netevd
cargo build && cargo test && cargo clippy -- -D warnings
```

See **[CONTRIBUTING.md](CONTRIBUTING.md)** for the full guide.

## License

[LGPL-3.0-or-later](https://www.gnu.org/licenses/lgpl-3.0.html) -- Copyright 2026 Susant Sahani <<ssahani@redhat.com>>
