<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Quick Wins

High-impact enhancements that can be completed in a few hours each. Start here if you want to contribute.

## Highest ROI (2-4 hours each)

### 1. Wire Event Filtering (4h)

The filter code exists in `src/filters/mod.rs` but is never used.

**What to do:** Add `filters: Vec<FilterConfig>` to the config struct, load filters in `main.rs`, pass them to listeners, apply before script execution.

```yaml
# New config section
filters:
  - name: "Only VPN interfaces"
    interface_pattern: "wg*"
    event_types: [routable, carrier]
    action: execute
  - name: "Ignore containers"
    interface_pattern: "veth*"
    action: ignore
```

### 2. Enable Real Metrics (4h)

The `Metrics` struct exists but counters are never incremented.

**What to do:** Create `Arc<Metrics>` in `main.rs`, pass to event handlers and script executor, increment counters on events/script runs/DBus calls. Connect `/metrics` endpoint to the real registry.

**Files:** `src/network/watcher.rs`, `src/system/execute.rs`, `src/api/handlers.rs`

### 3. Activate Audit Logging (3h)

`AuditLogger` exists but is never called.

**What to do:** Add `audit` config section, create logger in `main.rs`, inject into API handlers, event listeners, and script executor. Log network state changes, script executions, API requests, config changes.

### 4. Add API Configuration (2h)

The API server runs but ignores config for bind address and port.

**What to do:** Add `ApiConfig` struct with `enabled`, `bind_address`, `port` fields. Pass to server startup, add validation, allow disabling entirely.

```yaml
api:
  enabled: true
  bind_address: "0.0.0.0"
  port: 9090
```

### 5. Environment Variable Overrides (3h)

Standard 12-factor app pattern for easier container deployment.

**What to do:** After loading YAML, check `NETEVD_LOG_LEVEL`, `NETEVD_BACKEND`, `NETEVD_API_PORT` etc. and override config values.

## Medium Effort (6-8 hours each)

### 6. Complete API Status Endpoint

Currently returns `uptime_seconds: 0` always. Track `Instant::now()` at startup and `AtomicU64` event counter.

### 7. Implement CLI HTTP Client

Add reqwest dependency, implement actual API calls for `get_status()`, `get_interfaces()`, etc. Format output as table/JSON/YAML.

### 8. Event History Storage

Add `VecDeque<Event>` with bounded size. Record events as they occur, serve from `/api/v1/events` endpoint.

## Feature Ideas (10-20 hours)

### 9. Script Output Capture

Capture stdout/stderr from scripts and include in audit logs. Helps debug failing scripts.

### 10. Webhook Notifications

```yaml
webhooks:
  - url: https://slack.com/webhooks/...
    events: [link_down, route_change]
```

Send HTTP POST notifications for events without writing scripts.

### 11. Configuration Validation Endpoint

`POST /api/v1/config/validate` -- submit YAML, get back validation errors before applying.

### 12. Docker Compose Example

Full monitoring stack: netevd + Prometheus + Grafana in a single `docker-compose.yml`.

## Where to Start

Pick any item from the "Highest ROI" section. Each one:
- Has existing infrastructure to build on
- Can be completed independently
- Has clear scope and success criteria
- Immediately improves the user experience

See [ROADMAP.md](ROADMAP.md) for the full feature plan and [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup.
