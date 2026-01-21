# netevd Feature Roadmap

This document outlines potential features and enhancements for future versions of netevd.

## 📊 Monitoring & Observability (v0.2.0)

### Prometheus Metrics
- [ ] Expose `/metrics` endpoint for Prometheus scraping
- [ ] Metrics to track:
  - Event processing latency (histogram)
  - Script execution duration (histogram)
  - Active network interfaces (gauge)
  - Event count by type (counter)
  - Routing rules configured (gauge)
  - Script failures (counter)
  - Memory and CPU usage
  - DBus connection health

### OpenTelemetry Integration
- [ ] Distributed tracing support
- [ ] Span creation for event processing pipeline
- [ ] Integration with Jaeger/Zipkin
- [ ] Custom trace attributes for network events

### Enhanced Logging
- [ ] Structured JSON logging option
- [ ] Log levels per module
- [ ] Audit trail for all network changes
- [ ] Log rotation and compression
- [ ] Remote syslog support

### Health Checks
- [ ] HTTP health endpoint (`/health`)
- [ ] Readiness and liveness probes (Kubernetes)
- [ ] DBus connection monitoring
- [ ] Netlink socket health checks
- [ ] Script execution monitoring

---

## 🌐 API & Control Plane (v0.3.0)

### REST API
```rust
GET    /api/v1/status              // Daemon status
GET    /api/v1/interfaces          // List all interfaces
GET    /api/v1/interfaces/:name    // Interface details
GET    /api/v1/routes              // Routing table
GET    /api/v1/rules               // Policy routing rules
POST   /api/v1/reload              // Reload configuration
GET    /api/v1/events              // Recent events (SSE)
```

### gRPC API
- [ ] Programmatic control interface
- [ ] Streaming events via gRPC
- [ ] Configuration management
- [ ] State queries
- [ ] Authentication with mTLS

### WebSocket
- [ ] Real-time event streaming to clients
- [ ] Subscribe to specific event types
- [ ] Filter events by interface
- [ ] JSON event serialization

### CLI Tool Enhancement
```bash
netevd status                    # Show daemon status
netevd list interfaces           # List interfaces
netevd show interface eth0       # Interface details
netevd show routes               # Routing table
netevd show rules                # Policy rules
netevd reload                    # Reload config
netevd events --follow           # Follow events
netevd test script ./test.sh     # Test script execution
```

---

## 🔄 Configuration Management (v0.2.0)

### Hot Reload
- [ ] Reload configuration without restart (SIGHUP)
- [ ] Graceful reconfiguration
- [ ] Validate config before applying
- [ ] Rollback on errors

### Dynamic Configuration
- [ ] Watch config file for changes (inotify)
- [ ] Environment variable expansion in config
- [ ] Include directive for modular configs
- [ ] Templating support (Handlebars/Tera)

### Configuration Validation
```bash
netevd validate /etc/netevd/netevd.yaml
netevd check-scripts /etc/netevd/
```

### Configuration Backends
- [ ] Consul integration
- [ ] etcd integration
- [ ] AWS Systems Manager Parameter Store
- [ ] Azure Key Vault
- [ ] HashiCorp Vault

---

## 🛣️ Advanced Routing Features (v0.3.0)

### IPv6 Enhancements
- [ ] IPv6 policy routing rules
- [ ] IPv6 source address selection
- [ ] Dual-stack interface handling
- [ ] IPv6 route preferences

### ECMP (Equal-Cost Multi-Path)
- [ ] Load balancing across multiple gateways
- [ ] Per-packet vs per-flow distribution
- [ ] Weight-based load distribution

### Advanced Routing
- [ ] VRF (Virtual Routing and Forwarding) support
- [ ] MPLS label operations
- [ ] Route metrics and preferences
- [ ] Blackhole/unreachable routes
- [ ] Multicast routing support

### QoS Integration
- [ ] Traffic shaping on state changes
- [ ] Priority queuing configuration
- [ ] Bandwidth limit enforcement
- [ ] DiffServ/DSCP marking

### VLAN Support
- [ ] Monitor VLAN interfaces
- [ ] Dynamic VLAN creation
- [ ] 802.1Q tagging
- [ ] VLAN routing rules

---

## 🔌 Network Protocol Integration (v0.4.0)

### WireGuard
- [ ] Detect WireGuard interface creation
- [ ] Configure routes for WireGuard tunnels
- [ ] Monitor peer connections
- [ ] Dynamic allowed-ips updates

### BGP Integration
- [ ] Inject routes into FRRouting/BIRD
- [ ] BGP route announcements
- [ ] Prefix advertisement on events
- [ ] Integration with ExaBGP

### LLDP Discovery
- [ ] Neighbor discovery
- [ ] Topology mapping
- [ ] CDP/LLDP parsing
- [ ] Port identification

### Bonding/Bridging
- [ ] Monitor bond status
- [ ] Active/backup failover detection
- [ ] Bridge membership changes
- [ ] LACP state monitoring

### Tunnel Management
- [ ] VXLAN tunnel creation
- [ ] GRE tunnel management
- [ ] IPsec integration
- [ ] Automatic tunnel endpoint discovery

---

## 🎯 Event Processing Enhancements (v0.2.0)

### Event Filtering
```yaml
filters:
  - match:
      interface: "eth*"
      event_type: "routable"
      ip_family: "ipv4"
    action: execute
  - match:
      interface: "docker*"
    action: ignore
```

### Conditional Execution
```yaml
routable:
  - script: /etc/netevd/routable.d/vpn.sh
    condition: "interface == 'wg0' && has_gateway"
  - script: /etc/netevd/routable.d/dns.sh
    condition: "dns_servers.count > 0"
```

### Event Aggregation
- [ ] Debouncing rapid events
- [ ] Batch processing
- [ ] Rate limiting per interface
- [ ] Event deduplication

### Retry Logic
- [ ] Automatic retry on script failure
- [ ] Exponential backoff
- [ ] Max retry count configuration
- [ ] Dead letter queue for failed events

### Plugin System
- [ ] Dynamic plugin loading
- [ ] Lua scripting for inline event processing
- [ ] WASM plugin support
- [ ] Python/JavaScript bindings

---

## 🔒 Security Enhancements (v0.3.0)

### Mandatory Access Control
- [ ] SELinux policy module
- [ ] AppArmor profile
- [ ] Seccomp filters (syscall whitelist)

### Enhanced Validation
- [ ] Script signature verification
- [ ] Checksum validation
- [ ] Allowed script directory enforcement
- [ ] Script timeout enforcement

### Authentication & Authorization
- [ ] mTLS for API access
- [ ] JWT token support
- [ ] RBAC (Role-Based Access Control)
- [ ] Audit logging of all API calls

### Secrets Management
- [ ] Integration with HashiCorp Vault
- [ ] Encrypted configuration values
- [ ] Secret rotation support
- [ ] Environment variable secrets

---

## ☸️ Cloud & Orchestration (v0.4.0)

### Kubernetes
- [ ] Kubernetes Operator
- [ ] CRD for network policies
- [ ] CNI plugin integration
- [ ] Pod network events
- [ ] Service mesh integration

### Cloud Provider Integration
- [ ] AWS VPC route table updates
- [ ] Azure route table management
- [ ] GCP Cloud Router integration
- [ ] Elastic IP association
- [ ] Security group updates

### Container Integration
- [ ] Docker network events
- [ ] Podman integration
- [ ] systemd-nspawn support
- [ ] LXC/LXD integration

### Service Discovery
- [ ] Consul service registration
- [ ] etcd service announcements
- [ ] mDNS/Avahi integration
- [ ] DNS-SD support

---

## 🏢 High Availability (v0.5.0)

### Clustering
- [ ] Leader election (Raft/etcd)
- [ ] State synchronization between nodes
- [ ] Distributed lock service
- [ ] Quorum-based decisions

### Failover
- [ ] Automatic failover detection
- [ ] VRRP/keepalived integration
- [ ] IP takeover on failure
- [ ] Health check driven failover

### Replication
- [ ] Configuration replication
- [ ] Event log streaming
- [ ] State database replication
- [ ] Multi-region support

---

## 📡 IPAM Integration (v0.3.0)

### IP Address Management
- [ ] NetBox integration
- [ ] phpIPAM support
- [ ] Automatic IP allocation
- [ ] DHCP reservation sync
- [ ] DNS record updates

### Subnet Management
- [ ] Subnet utilization tracking
- [ ] Automatic subnet assignment
- [ ] CIDR calculation helpers
- [ ] IP conflict detection

---

## 🧪 Testing & Debugging (v0.2.0)

### Dry-Run Mode
```bash
netevd --dry-run  # Show what would happen without executing
```

### Event Simulation
```bash
netevd simulate routable eth0 --ip 192.168.1.100
netevd replay /var/log/netevd/events.log
```

### Testing Framework
- [ ] Integration test suite
- [ ] Network namespace testing
- [ ] Mock DBus server
- [ ] Chaos engineering support (random failures)
- [ ] Performance benchmarking suite

### Debugging Tools
- [ ] Event tracing (eBPF)
- [ ] Network topology visualization
- [ ] State dump command
- [ ] Performance profiling mode

---

## 🖥️ User Interface (v0.4.0)

### Web Dashboard
- [ ] Real-time interface status
- [ ] Event log viewer
- [ ] Configuration editor
- [ ] Routing table visualization
- [ ] Metrics graphs
- [ ] Script management

### TUI (Terminal UI)
```bash
netevd tui  # Launch interactive terminal UI
```
- [ ] htop-style interface monitoring
- [ ] Live event stream
- [ ] Interactive configuration
- [ ] Log viewer

---

## 🌍 Internationalization (v0.5.0)

- [ ] Multi-language log messages
- [ ] Localized documentation
- [ ] i18n support in web UI
- [ ] Error message translations

---

## 📦 Distribution & Packaging

### Additional Packages
- [ ] Alpine Linux (apk)
- [ ] Gentoo ebuild
- [ ] NixOS package
- [ ] FreeBSD port
- [ ] OpenWrt package
- [ ] Snap package
- [ ] AppImage

### Container Images
- [ ] Official Docker image
- [ ] Podman-ready images
- [ ] Multi-arch support (amd64, arm64, armv7)
- [ ] Distroless variants
- [ ] Alpine-based minimal images

---

## 🔧 Developer Experience

### SDK/Libraries
- [ ] Python bindings (PyO3)
- [ ] Go client library
- [ ] JavaScript/TypeScript SDK
- [ ] Rust client library

### Documentation
- [ ] Interactive API documentation (Swagger)
- [ ] Video tutorials
- [ ] Architecture deep-dives
- [ ] Performance tuning guide
- [ ] Security hardening guide

### Tooling
- [ ] Configuration generator
- [ ] Migration tool from other solutions
- [ ] Script template generator
- [ ] Visual routing policy designer

---

## 📊 Analytics & Reporting

### Reporting
- [ ] Daily/weekly network change reports
- [ ] Uptime statistics
- [ ] Interface utilization trends
- [ ] Event frequency analysis
- [ ] Performance regression detection

### Data Export
- [ ] Export events to InfluxDB
- [ ] TimescaleDB integration
- [ ] Elasticsearch logging
- [ ] Splunk forwarder
- [ ] Kafka event streaming

---

## 🚀 Performance Optimizations

### Zero-Copy Operations
- [ ] io_uring for file operations
- [ ] Zero-copy netlink operations
- [ ] Shared memory for IPC

### Caching
- [ ] Interface state caching
- [ ] Route table caching
- [ ] DNS lookup caching
- [ ] Configuration caching

### Parallelization
- [ ] Parallel script execution
- [ ] Thread pool for blocking operations
- [ ] SIMD optimizations
- [ ] Lock-free data structures

---

## Version Timeline

| Version | Target Date | Focus Areas |
|---------|-------------|-------------|
| **v0.2.0** | Q2 2026 | Monitoring, API, Hot-reload |
| **v0.3.0** | Q3 2026 | Advanced Routing, Security, IPAM |
| **v0.4.0** | Q4 2026 | Kubernetes, Cloud, UI |
| **v0.5.0** | Q1 2027 | HA, Clustering, Scale |

---

## Community Requests

Submit feature requests via GitHub Issues with the `enhancement` label.

**Priority will be given to:**
- Features with clear use cases
- Community-contributed implementations
- Security improvements
- Performance optimizations

---

## Contributing

Interested in implementing any of these features? See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**High-impact, beginner-friendly features:**
- CLI tool enhancements
- Additional script examples
- Documentation improvements
- Test coverage expansion
- Prometheus metrics

---

## License

All features will maintain LGPL-3.0-or-later licensing.
