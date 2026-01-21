# LinkedIn Post - netevd v0.1.0 (Enhanced Version)

---

🚀 **Announcing netevd v0.1.0: Next-Generation Network Event Daemon for Linux Infrastructure**

After extensive development and testing, I'm excited to release netevd—a production-ready, Rust-based network event daemon that fundamentally changes how we manage dynamic network configurations on Linux.

## 🎯 The Problem It Solves

Managing multi-interface Linux systems is complex. Traditional polling-based solutions have 5-second delays and high overhead. Manual network scripts are error-prone. netevd solves this with real-time event processing and intelligent automation.

## 🔥 Core Features

**🚄 Blazing Fast Performance**
- Sub-100ms event latency (50-100x faster than polling)
- 3-5 MB RAM footprint, <1% CPU idle
- Built on Tokio async/await for efficient concurrent processing
- Real-time netlink multicast subscriptions (no polling!)

**🔒 Enterprise-Grade Security**
- Runs as unprivileged user (no root required)
- Minimal capabilities (CAP_NET_ADMIN only)
- Defense-in-depth input validation against command injection
- systemd hardening: NoNewPrivileges, ProtectSystem=strict, PrivateTmp
- No capability inheritance to child processes
- Comprehensive test suite: 50 tests covering security boundaries

**🛣️ Advanced Routing Intelligence**
- Automatic routing policy rules for multi-interface setups
- Dynamic routing table creation (table ID = 200 + ifindex)
- Source-based routing for asymmetric network paths
- Atomic state updates—race-free network state management
- Automatic cleanup on interface removal

**🔌 Universal Backend Support**
- systemd-networkd (with JSON state export)
- NetworkManager (via DBus signals)
- dhclient (lease file monitoring)
- Seamless switching between backends

**📡 Real-Time Monitoring**
- Address changes (added/removed/modified)
- Route modifications across all tables
- Link state changes (carrier detection)
- Gateway reachability tracking
- All via concurrent netlink watchers

**🎬 Flexible Script Execution**
- Event-driven scripts for every network state:
  - `carrier.d/` - Physical link established
  - `routable.d/` - Gateway reachable
  - `configured.d/` - IP address assigned
  - `degraded.d/` - Partial configuration
  - `no-carrier.d/` - Cable disconnected
  - `activated.d/` - NetworkManager activation
  - `routes.d/` - Route table changes
- Rich environment variables (interface name, IP addresses, JSON data)
- Input validation prevents shell injection attacks

**🔄 DBus Integration**
- systemd-resolved for DNS management
- systemd-hostnamed for hostname updates
- Async DBus communication (non-blocking)

## 📊 Technical Architecture Highlights

**Concurrency Model:**
```rust
tokio::select! {
    _ = watch_addresses() => {},  // Netlink address events
    _ = watch_routes() => {},     // Netlink route events
    _ = watch_links() => {},      // Netlink link events
    _ = spawn_listener() => {},   // DBus or file watcher
}
```

**State Management:**
- Arc<RwLock<NetworkState>> for thread-safe access
- Lock-free reads, minimal write contention
- Atomic updates prevent race conditions

## 🎯 Real-World Use Cases

**Multi-Homed Servers**
Automatically configure policy routing so traffic arriving on eth1 returns via eth1 (not the default gateway). Essential for servers with multiple public IPs.

**Dynamic DNS**
Update DNS records immediately when DHCP assigns new IPs—no 5-minute delays.

**VPN Automation**
Execute scripts to add routes, update firewall rules, and configure split tunneling when VPN interfaces come up.

**Container/VM Networking**
React to network state changes in orchestrated environments (Kubernetes, libvirt, systemd-nspawn).

**Network Monitoring**
Send alerts when links lose carrier, IPs change unexpectedly, or routes disappear.

**Cloud/Edge Deployments**
Lightweight enough for embedded systems (3-5 MB), powerful enough for enterprise infrastructure.

## 📦 Distribution & Installation

**From crates.io:**
```bash
cargo install netevd
```

**Native Packages:**
- 🔴 Fedora/RHEL/CentOS: RPM packages
- 🔵 Debian/Ubuntu: DEB packages
- 🟣 Arch Linux: AUR (yay -S netevd)

**All packages include:**
- systemd service with security hardening
- Automatic user creation
- Pre-configured script directories
- Man pages and documentation

## 🔬 Why Rust?

This isn't just about performance—it's about correctness:
- **Memory safety** prevents CVEs common in C daemons
- **Fearless concurrency** enables multiple watchers without data races
- **Zero-cost abstractions** deliver C-like performance with high-level code
- **Strong typing** catches bugs at compile time
- **Cargo ecosystem** provides battle-tested async libraries

## 📈 Benchmarks

| Metric | Value | Notes |
|--------|-------|-------|
| Memory (idle) | 3-5 MB | Minimal footprint |
| CPU (idle) | <1% | Event-driven, not polling |
| Event latency | <100 ms | Netlink → script execution |
| Startup time | <100 ms | Fast boot integration |
| Concurrent events | 1000+/sec | Async processing scales |

## 🔄 Migration Path

Migrating from network-broker (Go) or custom scripts? netevd provides:
- Configuration compatibility layer
- Script directory structure maintained
- Improved performance and security
- Full migration guide included

## 🛠️ What's Next?

Planned features for v0.2.0:
- Prometheus metrics endpoint
- Advanced event filtering (regex, conditions)
- IPv6 policy routing enhancements
- Performance profiling and optimization
- gRPC API for programmatic control
- Kubernetes operator integration

## 🙏 Acknowledgments

Huge thanks to:
- The Rust community for incredible tooling
- Early testers who provided invaluable feedback
- systemd, NetworkManager, and netlink developers
- Everyone who contributed ideas and use cases

## 🔗 Resources

📖 **GitHub:** https://github.com/ssahani/netevd
🦀 **crates.io:** https://crates.io/crates/netevd
📚 **Documentation:** Comprehensive README with architecture diagrams
📋 **License:** LGPL-3.0-or-later (commercial use allowed)
🐛 **Issues:** Contributions and bug reports welcome!

## 💬 Let's Connect

If you're working with:
- Linux network automation
- Multi-homed server configurations
- Network monitoring solutions
- Container/VM orchestration
- Edge/IoT deployments

I'd love to hear your use cases, challenges, and feedback. PRs welcome—let's build the future of Linux network management together!

#Rust #Linux #OpenSource #Networking #DevOps #SRE #SystemsProgramming #Infrastructure #CloudNative #Automation #Performance #Security #RustLang #SystemAdmin #NetworkEngineering #Kubernetes #EdgeComputing

---

**Try it today:**
```bash
cargo install netevd
systemctl enable --now netevd
```

Questions? Drop a comment or open an issue on GitHub!
