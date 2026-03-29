<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Changelog

Based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Rewrote all documentation for clarity and conciseness

## [0.2.0] - 2026-01-21

### Added

**CLI tools** -- `netevd status`, `list`, `show`, `events`, `reload`, `validate`, `test` commands with JSON/YAML/table output formats and dry-run mode.

**REST API** -- 9 endpoints built on Axum: status, interfaces, routes, rules, events, reload, health, and Prometheus metrics. WebSocket support planned.

**Prometheus metrics** -- 15+ metrics across 6 categories (daemon, interfaces, routing, scripts, DBus, netlink) with histogram buckets for latency tracking.

**Audit logging** -- Structured JSON audit logs for compliance and debugging.

**IPv6 support** -- Policy routing for IPv6 with RFC 6724 address selection and dual-stack operations.

**Event filtering** -- Pattern matching and conditional expressions for filtering noisy events.

**Cloud integration** -- AWS EC2, Azure IMDS, and GCP metadata API support (experimental).

**Kubernetes** -- CRD definitions, DaemonSet patterns, ConfigMap integration. Docker images for Debian (~150MB) and Alpine (~50MB).

**Packaging** -- Published to crates.io. RPM, DEB, and AUR packages available.

### Changed
- Improved performance and memory efficiency
- Enhanced error messages and logging
- Refactored internal architecture for modularity

### Fixed
- Race conditions in concurrent event processing
- Memory leaks in long-running sessions
- Configuration reload edge cases
- IPv6 routing rule creation issues

## [0.1.0] - 2025-12-15

### Added

**Core** -- Async event daemon built on Tokio with support for systemd-networkd (DBus), NetworkManager (DBus), and dhclient (lease file monitoring via inotify).

**Event monitoring** -- Real-time address, link, and route change detection via netlink multicast with sub-100ms latency.

**Script execution** -- Event-triggered scripts in 9 directories (`carrier.d/`, `routable.d/`, `routes.d/`, etc.) with environment variables for network state. Input validation prevents command injection.

**Routing policy rules** -- Automatic per-interface routing tables for multi-homed servers with source-based and destination-based rules.

**Security** -- Privilege separation (root -> netevd user), minimal capabilities (CAP_NET_ADMIN only), systemd hardening (NoNewPrivileges, ProtectSystem=strict, PrivateTmp).

**DBus integration** -- systemd-resolved (DNS) and systemd-hostnamed (hostname) integration for dhclient backend.

**Configuration** -- YAML-based config with per-interface monitoring, flexible routing policy rules, backend selection, and logging control.

## Upgrade Guide

### 0.1.0 -> 0.2.0

No breaking changes. Existing configurations work without modification. New features are opt-in:

```yaml
# Optional: enable API
api:
  enabled: true
  bind: "127.0.0.1:8080"

# Optional: enable metrics
metrics:
  enabled: true

# Optional: event filters
filters:
  - interface: "!docker*"
```

Upgrade steps:

```bash
sudo systemctl stop netevd
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo systemctl start netevd
netevd --version    # verify
```
