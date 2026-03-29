<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Roadmap

## Current State

Based on codebase analysis: ~70% of advertised features are fully implemented, ~20% are partially implemented (infrastructure exists but isn't wired up), and ~10% are documented but missing.

## Critical (v0.2.1) -- Fix What's Advertised

### Complete API handlers

8 of 9 endpoints return stub data. Track real uptime, count events, query actual interface state, implement route/rule listing, event history, config reload, and real health checks.

**Files:** `src/api/handlers.rs`

### Implement CLI commands

All `netevd status/list/show/events` commands are stubs. Add HTTP client (reqwest), implement API calls, add event streaming and output formatting.

**Files:** `src/cli/handler.rs`

### Fix IPv6 routing

IPv6 policy routing rules don't work. Either implement via rtnetlink or fall back to `ip -6 rule add` commands.

**Files:** `src/network/ipv6.rs`

### Cloud integration decision

AWS/Azure/GCP modules are all stubs. Options: complete them, mark as experimental behind feature flags, or remove from docs until implemented.

**Recommendation:** Feature-gate and mark experimental.

## High Priority (v0.3.0) -- Wire Existing Code

### Event filtering

Code exists in `src/filters/mod.rs` but is never called. Add `filters` config section, pass filters to listeners, apply before script execution.

### Metrics collection

Prometheus metrics struct exists but counters are never incremented. Wire into event handlers, script executor, DBus/netlink operations. Connect `/metrics` endpoint to real registry.

### Audit logging

Audit logger exists but is never called. Add config section, inject into API handlers, event listeners, and script executor.

### API configuration

API server runs but ignores config. Add `api` config section with `enabled`, `bind_address`, `port` options.

## Medium Priority (v0.4.0)

- WebSocket event streaming for real-time web UIs
- API rate limiting via tower-governor
- TLS/SSL support via rustls
- Dynamic config reload (SIGHUP + POST endpoint)

## Low Priority (v0.5.0+)

- Plugin architecture (Rust trait-based)
- OpenTelemetry distributed tracing
- Web dashboard UI
- Kubernetes operator (CRD exists, operator logic missing)

## Community Requests

- systemd socket activation
- JSON output for all CLI commands
- Configuration file includes
- Script output capture and logging
- Email/webhook notifications
- BGP route announcements
- VRRP/keepalived integration

## Release Schedule

| Version | Focus | Target |
|---------|-------|--------|
| v0.2.1 | Fix stubs (API, CLI, IPv6) | Feb 2026 |
| v0.3.0 | Wire existing code (filters, metrics, audit) | Mar 2026 |
| v0.4.0 | WebSocket, rate limiting, TLS | May 2026 |
| v0.5.0 | Plugins, dashboard | Jul 2026 |
| v1.0.0 | Production-ready | Q4 2026 |

## Contributing

The best places to help right now:

1. **Critical fixes** -- Implement stub API/CLI handlers
2. **Wire existing code** -- Connect filters, metrics, audit logging
3. **Tests** -- Increase coverage, especially for API and CLI
4. **Documentation** -- Improve examples and guides

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions.
