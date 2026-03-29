<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# REST API Reference

netevd exposes an HTTP API built on [Axum](https://github.com/tokio-rs/axum) for monitoring and management.

## Configuration

```yaml
# /etc/netevd/netevd.yaml
api:
  enabled: true
  bind_address: "127.0.0.1"    # localhost only (recommended)
  port: 9090
```

Verify: `curl http://localhost:9090/health`

**Security note:** The API has no authentication. Bind to localhost and use a reverse proxy (nginx, Apache) with auth for remote access, or restrict via firewall rules.

## Endpoints

### GET /api/v1/status

Daemon status and statistics.

```bash
curl http://localhost:9090/api/v1/status
```

```json
{
  "status": "ok",
  "version": "0.2.0",
  "uptime_seconds": 86400,
  "backend": "systemd-networkd",
  "interfaces_monitored": 3,
  "events_processed": 1542,
  "scripts_executed": 89,
  "last_event": "2026-01-23T10:30:45Z"
}
```

### GET /api/v1/interfaces

List monitored interfaces. Filter with `?state=up` or `?type=ethernet`.

```bash
curl http://localhost:9090/api/v1/interfaces
curl "http://localhost:9090/api/v1/interfaces?state=up"
```

```json
{
  "interfaces": [
    {
      "name": "eth0",
      "index": 2,
      "state": "up",
      "type": "ethernet",
      "mac_address": "00:11:22:33:44:55",
      "mtu": 1500,
      "addresses": [
        {"ip": "192.168.1.100", "prefix": 24, "family": "ipv4", "scope": "global"}
      ],
      "flags": ["UP", "BROADCAST", "RUNNING", "MULTICAST"]
    }
  ],
  "count": 1
}
```

### GET /api/v1/interfaces/:name

Detailed info for a specific interface.

```bash
curl http://localhost:9090/api/v1/interfaces/eth0
```

Returns the interface object with additional `routes`, `dns_servers`, and `statistics` fields.

### GET /api/v1/routes

Routing table entries. Filter with `?interface=eth0`, `?table=203`, or `?family=ipv4`.

```bash
curl http://localhost:9090/api/v1/routes
curl "http://localhost:9090/api/v1/routes?table=203"
```

```json
{
  "routes": [
    {
      "destination": "0.0.0.0/0",
      "gateway": "192.168.1.1",
      "interface": "eth0",
      "metric": 100,
      "table": "main",
      "protocol": "boot",
      "scope": "universe"
    }
  ],
  "count": 1
}
```

### GET /api/v1/rules

Routing policy rules managed by netevd.

```bash
curl http://localhost:9090/api/v1/rules
```

```json
{
  "rules": [
    {
      "priority": 32765,
      "from": "192.168.1.100/32",
      "to": null,
      "table": 203,
      "action": "lookup",
      "interface": "eth1"
    }
  ],
  "count": 1
}
```

### GET /api/v1/events

Recent network events. Parameters: `?limit=100`, `?interface=eth0`, `?event_type=address`, `?since=2026-01-23T10:00:00Z`.

```bash
curl http://localhost:9090/api/v1/events
curl "http://localhost:9090/api/v1/events?interface=eth0&limit=50"
```

```json
{
  "events": [
    {
      "id": "evt-12345",
      "timestamp": "2026-01-23T10:30:45Z",
      "event_type": "address",
      "interface": "eth0",
      "action": "added",
      "details": {"address": "192.168.1.100/24", "family": "ipv4"}
    }
  ],
  "count": 1,
  "has_more": false
}
```

### POST /api/v1/reload

Reload configuration without restarting.

```bash
curl -X POST http://localhost:9090/api/v1/reload
```

```json
{
  "status": "success",
  "message": "Configuration reloaded successfully",
  "timestamp": "2026-01-23T10:35:00Z"
}
```

### GET /health

Health check for load balancers and monitoring.

```bash
curl http://localhost:9090/health
```

Returns `200` when healthy, `503` when degraded.

```json
{
  "status": "healthy",
  "checks": {"netlink": "ok", "dbus": "ok", "config": "ok", "backend": "ok"}
}
```

### GET /metrics

Prometheus metrics in text exposition format.

```bash
curl http://localhost:9090/metrics
```

```
# HELP netevd_uptime_seconds Time since netevd started
# TYPE netevd_uptime_seconds gauge
netevd_uptime_seconds 86400

# HELP netevd_events_total Total network events processed
# TYPE netevd_events_total counter
netevd_events_total{type="address",interface="eth0"} 45
```

See [METRICS.md](METRICS.md) for the full metrics reference.

## Error Responses

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Interface 'eth99' not found",
    "details": {"available_interfaces": ["eth0", "eth1"]}
  }
}
```

| HTTP Code | Meaning |
|-----------|---------|
| 200 | Success |
| 400 | Invalid parameters |
| 404 | Resource not found |
| 429 | Rate limited (when enabled) |
| 500 | Internal error |
| 503 | Health check failed |

## Prometheus Integration

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'netevd'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Example: Monitoring Script

```bash
#!/bin/bash
API="http://localhost:9090/api/v1"

echo "=== Status ==="
curl -s "$API/status" | jq '.status, .uptime_seconds, .events_processed'

echo "=== Interfaces ==="
curl -s "$API/interfaces" | jq -r '.interfaces[] | "\(.name): \(.state) - \(.addresses[0].ip)"'

echo "=== Recent Events ==="
curl -s "$API/events?limit=5" | jq -r '.events[] | "\(.timestamp) [\(.event_type)] \(.interface): \(.action)"'
```

## See Also

- [Metrics Reference](METRICS.md) -- all Prometheus metrics
- [Configuration](../CONFIGURATION.md) -- API config options
- [Architecture](ARCHITECTURE.md) -- API internals
