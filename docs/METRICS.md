<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Prometheus Metrics

netevd exposes metrics at `GET /metrics` in Prometheus text exposition format.

## Setup

### Enable in netevd

```yaml
# /etc/netevd/netevd.yaml
metrics:
  enabled: true
  endpoint: "/metrics"
```

### Prometheus scrape config

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'netevd'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Verify

```bash
curl http://localhost:9090/metrics
```

## Metric Reference

### Daemon

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `netevd_uptime_seconds` | Gauge | -- | Seconds since daemon started |
| `netevd_events_total` | Counter | `type`, `interface`, `backend` | Total events processed |
| `netevd_event_duration_seconds` | Histogram | `type` | Event processing time |

### Interfaces

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `netevd_interfaces_total` | Gauge | -- | Number of monitored interfaces |
| `netevd_interface_state_changes_total` | Counter | `interface`, `state` | State transitions |

### Routing

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `netevd_routing_rules_total` | Gauge | -- | Active policy rules |
| `netevd_routes_total` | Gauge | `table` | Routes in custom tables |

### Scripts

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `netevd_script_executions_total` | Counter | `script`, `event_type` | Total script runs |
| `netevd_script_duration_seconds` | Histogram | `script` | Script execution time |
| `netevd_script_failures_total` | Counter | `script`, `exit_code` | Failed executions |

### DBus

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `netevd_dbus_calls_total` | Counter | `service`, `method` | DBus method calls |
| `netevd_dbus_errors_total` | Counter | -- | DBus errors |

### Netlink

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `netevd_netlink_messages_total` | Counter | `message_type` | Messages processed |
| `netevd_netlink_errors_total` | Counter | -- | Netlink errors |

## Useful PromQL Queries

```promql
# Event rate per second
rate(netevd_events_total[5m])

# Events by type
sum by (type) (rate(netevd_events_total[5m]))

# Top 5 most active interfaces
topk(5, sum by (interface) (rate(netevd_events_total[5m])))

# P95 event processing latency
histogram_quantile(0.95, rate(netevd_event_duration_seconds_bucket[5m]))

# Script failure rate
rate(netevd_script_failures_total[5m])

# Slow scripts (P95 > 1s)
histogram_quantile(0.95, rate(netevd_script_duration_seconds_bucket[5m])) > 1

# Flapping interfaces (high up/down rate)
sum by (interface) (rate(netevd_interface_state_changes_total{state=~"up|down"}[5m])) > 0.1
```

## Alert Rules

```yaml
# netevd_alerts.yml
groups:
  - name: netevd
    interval: 30s
    rules:
      - alert: NetevdDown
        expr: up{job="netevd"} == 0
        for: 1m
        labels: {severity: critical}
        annotations:
          summary: "netevd is not responding on {{ $labels.instance }}"

      - alert: HighEventLatency
        expr: histogram_quantile(0.95, rate(netevd_event_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels: {severity: warning}
        annotations:
          summary: "P95 event latency {{ $value }}s (threshold: 1s)"

      - alert: InterfaceFlapping
        expr: sum by (interface) (rate(netevd_interface_state_changes_total[5m])) > 0.1
        for: 10m
        labels: {severity: warning}
        annotations:
          summary: "{{ $labels.interface }} flapping: {{ $value }} changes/sec"

      - alert: ScriptFailures
        expr: rate(netevd_script_failures_total[5m]) > 0
        for: 5m
        labels: {severity: warning}
        annotations:
          summary: "Script failures: {{ $value }}/sec"

      - alert: DbusErrors
        expr: rate(netevd_dbus_errors_total[5m]) > 0
        for: 5m
        labels: {severity: warning}
        annotations:
          summary: "DBus errors: {{ $value }}/sec"
```

Load in Prometheus:

```yaml
rule_files:
  - "netevd_alerts.yml"
```

## Grafana Dashboard

### Panels

| Row | Panels |
|-----|--------|
| Daemon Status | Uptime (stat), Events Processed (stat), Scripts Executed (stat) |
| Event Processing | Event Rate by Type (graph), P95 Latency (graph) |
| Interfaces | Interface Count (stat), State Changes (graph) |
| Scripts | Script Duration Heatmap, Script Failures by Name (graph) |
| System Health | DBus Calls (graph), Netlink Messages (graph) |

### Sample panel config

```json
{
  "title": "Event Rate by Type",
  "type": "graph",
  "targets": [
    {
      "expr": "sum(rate(netevd_events_total[5m])) by (type)",
      "legendFormat": "{{type}}"
    }
  ]
}
```

## Recording Rules

Pre-compute expensive queries:

```yaml
groups:
  - name: netevd_recording
    interval: 30s
    rules:
      - record: netevd:event_rate:5m
        expr: sum(rate(netevd_events_total[5m]))
      - record: netevd:script_p95:5m
        expr: histogram_quantile(0.95, rate(netevd_script_duration_seconds_bucket[5m]))
```

## Performance Impact

Metrics add negligible overhead: ~0.5% CPU, ~2-3 MB memory, ~5ms latency. Use 15-30s scrape intervals and keep label cardinality under 1000.

## See Also

- [API Reference](API.md) -- `/metrics` endpoint details
- [Architecture](ARCHITECTURE.md) -- metrics internals
