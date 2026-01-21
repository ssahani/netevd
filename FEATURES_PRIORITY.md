# netevd - Priority Feature Implementation Guide

Quick reference for high-impact features to implement next.

## 🎯 Top 10 High-Impact Features

### 1. **Prometheus Metrics Endpoint** (Highest Priority)
**Why:** Essential for production monitoring
**Effort:** Medium
**Impact:** Very High

```rust
// metrics to expose:
- netevd_events_total{type, interface, backend}
- netevd_event_duration_seconds{type}
- netevd_script_duration_seconds{script}
- netevd_script_failures_total{script}
- netevd_interfaces_total{state}
- netevd_routing_rules_total
```

**Implementation:**
- Use `prometheus` crate
- Expose on configurable port (default: 9090)
- Add `/metrics` HTTP endpoint

---

### 2. **REST API for Status Queries**
**Why:** Enable programmatic control and monitoring
**Effort:** Medium
**Impact:** High

```rust
// Endpoints:
GET /api/v1/status              // Overall daemon status
GET /api/v1/interfaces          // All interfaces
GET /api/v1/interfaces/:name    // Specific interface
GET /api/v1/routes              // Routing table
GET /api/v1/rules               // Policy rules
POST /api/v1/reload             // Reload config
GET /api/v1/events              // Event stream (SSE)
```

**Tech Stack:**
- `axum` or `actix-web` for HTTP server
- `tower` for middleware
- Optional: mTLS authentication

---

### 3. **Configuration Hot-Reload (SIGHUP)**
**Why:** No downtime for config changes
**Effort:** Low
**Impact:** High

```rust
// Handle SIGHUP signal
signal_hook::iterator::Signals::new(&[SIGHUP])?
// Reload /etc/netevd/netevd.yaml
// Validate before applying
// Graceful transition
```

---

### 4. **Event Filtering & Conditional Execution**
**Why:** Users need fine-grained control
**Effort:** Medium
**Impact:** High

```yaml
filters:
  - match:
      interface_pattern: "eth*"
      event_type: "routable"
      ip_family: "ipv4"
    action: execute
    scripts:
      - /etc/netevd/routable.d/custom.sh

  - match:
      interface_pattern: "docker*"
    action: ignore

  - match:
      interface: "wg0"
      condition: "has_gateway && dns_count > 0"
    action: execute
```

---

### 5. **Enhanced CLI Tool**
**Why:** Better user experience and debugging
**Effort:** Low-Medium
**Impact:** Medium-High

```bash
netevd status                    # Daemon status
netevd list interfaces           # List all interfaces
netevd show interface eth0       # Detailed info
netevd show routes               # Routing table
netevd show rules                # Policy rules
netevd events --follow           # Live event stream
netevd test script ./test.sh     # Test script execution
netevd validate config.yaml      # Validate config
netevd reload                    # Trigger reload
```

---

### 6. **IPv6 Policy Routing Support**
**Why:** IPv6 is increasingly important
**Effort:** Medium
**Impact:** Medium

- IPv6 routing policy rules
- Source address selection
- Dual-stack handling
- IPv6-specific route tables

---

### 7. **WireGuard Integration**
**Why:** VPN usage is very common
**Effort:** Low-Medium
**Impact:** Medium-High

- Detect WireGuard interfaces automatically
- Configure routes for WireGuard tunnels
- Monitor peer connections
- Execute scripts on tunnel up/down

---

### 8. **Web Dashboard (Basic)**
**Why:** Visual monitoring is valuable
**Effort:** High
**Impact:** Medium

Features:
- Real-time interface status
- Live event log
- Routing table visualization
- Basic configuration editing
- Metrics graphs

**Tech Stack:**
- Backend: Same as REST API
- Frontend: HTMX or Alpine.js (lightweight)
- WebSocket for real-time updates

---

### 9. **Event Debouncing/Aggregation**
**Why:** Prevent script flooding on rapid changes
**Effort:** Low
**Impact:** Medium

```yaml
debounce:
  enabled: true
  window: 5s        # Wait 5s after first event
  max_events: 10    # Or until 10 events
```

Prevents scripts from running 100 times when interface flaps.

---

### 10. **Kubernetes Operator**
**Why:** Cloud-native deployment support
**Effort:** High
**Impact:** High (for K8s users)

- CRD for NetworkPolicy
- Automatic deployment
- Pod network event handling
- Integration with CNI plugins

---

## 🚀 Quick Wins (Low Effort, High Impact)

### A. **Script Timeout Enforcement**
```yaml
script_timeout: 30s  # Kill scripts after 30 seconds
```

### B. **Script Exit Code Logging**
Log and expose metrics on script failures:
```
2026-01-21 netevd: Script /etc/netevd/routable.d/vpn.sh exited with code 1
```

### C. **JSON Log Output Option**
```yaml
system:
  log_format: "json"  # or "text"
```

### D. **Environment Variable Expansion in Config**
```yaml
network:
  links: "${MONITOR_INTERFACES:-eth0 eth1}"
```

### E. **Dry-Run Mode**
```bash
netevd --dry-run  # Show what would happen without executing
```

### F. **Config Include Directive**
```yaml
include:
  - /etc/netevd/conf.d/*.yaml
```

---

## 🔒 Security Quick Wins

### A. **SELinux Policy Module**
Provide ready-to-use SELinux policy for Fedora/RHEL.

### B. **AppArmor Profile**
Provide profile for Ubuntu/Debian systems.

### C. **Script Signature Verification**
```yaml
security:
  verify_scripts: true
  allowed_signers:
    - fingerprint: "ABC123..."
```

### D. **Seccomp Filters**
Restrict syscalls to minimum required set.

---

## 📊 Monitoring Quick Wins

### A. **Health Check Endpoint**
```
GET /health
{
  "status": "healthy",
  "uptime": 86400,
  "interfaces": 3,
  "routing_rules": 6
}
```

### B. **Structured Logging**
Add context to all log entries:
```json
{
  "timestamp": "2026-01-21T10:30:00Z",
  "level": "info",
  "interface": "eth0",
  "event": "routable",
  "ip": "192.168.1.100"
}
```

---

## 🎯 Implementation Priorities by Use Case

### For Production Deployments
1. Prometheus metrics
2. Health checks
3. Hot reload
4. Event debouncing
5. Script timeouts

### For Enterprise Users
1. REST API
2. IPv6 support
3. Event filtering
4. Web dashboard
5. Audit logging

### For Cloud/Kubernetes Users
1. Kubernetes operator
2. Prometheus metrics
3. Health checks
4. REST API
5. Container image

### For Home/Small Office
1. WireGuard integration
2. Enhanced CLI
3. Web dashboard
4. IPv6 support
5. Documentation

---

## 📅 Suggested v0.2.0 Feature Set

**Theme:** Monitoring & Control

1. ✅ Prometheus metrics endpoint
2. ✅ REST API (read-only endpoints)
3. ✅ Configuration hot-reload (SIGHUP)
4. ✅ Enhanced CLI tool
5. ✅ Script timeout enforcement
6. ✅ Health check endpoint
7. ✅ JSON logging option
8. ✅ Event debouncing
9. ✅ Dry-run mode
10. ✅ Config validation command

**Target:** 6-8 weeks development time

---

## 💡 Community Contribution Ideas

**Good First Issues:**
- Add example scripts for common use cases
- Write integration tests
- Improve documentation
- Add Ansible role
- Create Docker compose examples

**Intermediate:**
- Implement CLI enhancements
- Add config validation
- Create systemd timer for cleanup
- Build web dashboard prototype

**Advanced:**
- Prometheus metrics implementation
- REST API development
- Kubernetes operator
- IPv6 routing enhancements
- eBPF-based monitoring

---

## 📞 Feedback Welcome

Which features are most important to you? Open a GitHub issue with:
- Feature request label
- Your use case
- Expected behavior
- Willingness to contribute

**High-impact features with community PRs will be prioritized!**
