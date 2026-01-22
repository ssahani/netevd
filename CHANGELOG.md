<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Fresh documentation structure
- Comprehensive installation guide
- Detailed configuration guide
- Contributing guidelines
- Security policy

## [0.2.0] - 2026-01-21

### Added

#### Developer Tools
- Enhanced CLI with comprehensive commands:
  - `status`: Display current network status
  - `list`: List all monitored interfaces
  - `show`: Show detailed interface information
  - `events`: Stream network events in real-time
  - `reload`: Reload configuration without restart
  - `validate`: Validate configuration files
  - `test`: Dry-run mode for testing changes
- Multiple output formats: JSON, YAML, and table
- Built-in configuration validation
- Dry-run mode for safe testing

#### Enterprise Features
- REST API built with Axum framework
  - 9 endpoints for network management
  - WebSocket support for real-time events
  - OpenAPI/Swagger documentation
- Advanced event filtering system
  - Pattern matching
  - Conditional expressions
  - Field-based filtering
- Prometheus metrics integration
  - 15+ metrics across 6 categories
  - Custom metric endpoints
  - Performance monitoring
- Structured JSON audit logging
  - Compliance tracking
  - Event correlation
  - Debugging support
- IPv6 support
  - Policy routing for IPv6
  - RFC 6724 address selection
  - Dual-stack operations
- Web dashboard
  - Real-time monitoring interface
  - Auto-refresh capabilities
  - Event visualization

#### Cloud & Kubernetes
- Kubernetes operator support
  - Custom Resource Definitions (CRDs)
  - DaemonSet deployment patterns
  - ConfigMap integration
- Docker images
  - Debian-based (~150MB)
  - Alpine-based (~50MB)
  - Multi-arch support
- Cloud provider integrations
  - AWS EC2 metadata support
  - Azure IMDS integration
  - GCP metadata API
- Distribution packages
  - Published to crates.io
  - RPM packages for Fedora/RHEL
  - DEB packages for Debian/Ubuntu
  - AUR package for Arch Linux

### Changed
- Improved performance and memory efficiency
- Enhanced error messages and logging
- Refactored internal architecture for modularity
- Updated dependencies to latest versions

### Fixed
- Race conditions in concurrent event processing
- Memory leaks in long-running sessions
- Configuration reload edge cases
- IPv6 routing rule creation issues

## [0.1.0] - 2025-12-15

### Added

#### Core Features
- Async/await architecture built on Tokio
- Multiple network manager support:
  - systemd-networkd via DBus
  - NetworkManager via DBus
  - dhclient via lease file monitoring
- Real-time network event monitoring:
  - Address changes (add/remove/modify)
  - Link changes (up/down)
  - Route modifications
- Automatic routing policy rules
  - Multi-interface routing
  - Custom routing tables
  - Source-based routing
- Script execution framework
  - Event-triggered scripts
  - Environment variable passing
  - Input validation

#### Security Features
- Privilege dropping
  - Starts as root, drops to unprivileged user
  - Minimal capability retention (CAP_NET_ADMIN only)
  - No capability inheritance to child processes
- Input validation
  - Defense against command injection
  - Sanitization of environment variables
  - Strict interface name validation
- systemd hardening
  - NoNewPrivileges
  - ProtectSystem=strict
  - PrivateTmp
  - Read-only filesystem where possible

#### Network Operations
- Netlink-based monitoring
  - Sub-100ms event latency
  - Multicast group subscriptions
  - Direct kernel communication
- DBus integration
  - systemd-resolved for DNS
  - systemd-hostnamed for hostname
  - NetworkManager state tracking
- Lease file parsing
  - dhclient lease monitoring
  - inotify-based file watching
  - Automatic refresh on changes

#### Configuration
- YAML-based configuration
- Per-interface monitoring control
- Flexible routing policy configuration
- Backend selection (networkd/NetworkManager/dhclient)
- Logging level control

#### Script Directories
- `carrier.d/` - Link carrier gained
- `no-carrier.d/` - Link carrier lost
- `configured.d/` - Interface configured
- `degraded.d/` - Interface degraded
- `routable.d/` - Interface routable
- `activated.d/` - NetworkManager activation
- `disconnected.d/` - NetworkManager disconnection
- `manager.d/` - Manager state changes
- `routes.d/` - Route changes

#### Documentation
- Comprehensive README with diagrams
- Usage examples
- Security model documentation
- Installation instructions
- Troubleshooting guide

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- Implemented defense-in-depth security model
- Input validation for all external data
- Capability-based privilege restriction
- systemd service hardening

## Version History Summary

| Version | Release Date | Key Features |
|---------|--------------|--------------|
| 0.2.0   | 2026-01-21   | Enterprise features, CLI, API, Kubernetes |
| 0.1.0   | 2025-12-15   | Initial release with core functionality |

## Upgrade Guide

### From 0.1.0 to 0.2.0

#### Breaking Changes
None. Version 0.2.0 is fully backward compatible with 0.1.0 configurations.

#### New Features to Consider
1. **CLI Commands**: Explore the new CLI for easier management
   ```bash
   netevd status
   netevd list
   netevd events
   ```

2. **REST API**: Enable API server for remote management
   ```yaml
   api:
     enabled: true
     bind: "127.0.0.1:8080"
   ```

3. **Metrics**: Add Prometheus monitoring
   ```yaml
   metrics:
     enabled: true
     endpoint: "/metrics"
   ```

4. **Event Filtering**: Filter noisy events
   ```yaml
   filters:
     - interface: "!docker*"
     - state: "routable"
   ```

#### Configuration Updates
No changes required to existing configurations. New features are opt-in.

#### Migration Steps
1. Backup configuration:
   ```bash
   sudo cp /etc/netevd/netevd.yaml /etc/netevd/netevd.yaml.backup
   ```

2. Stop service:
   ```bash
   sudo systemctl stop netevd
   ```

3. Update binary:
   ```bash
   sudo install -Dm755 target/release/netevd /usr/bin/netevd
   ```

4. Start service:
   ```bash
   sudo systemctl start netevd
   ```

5. Verify:
   ```bash
   sudo systemctl status netevd
   netevd --version
   ```

## Development Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

## Release Process

1. Update CHANGELOG.md with new version
2. Update version in Cargo.toml
3. Run tests: `cargo test`
4. Build release: `cargo build --release`
5. Create git tag: `git tag -a v0.X.Y -m "Release v0.X.Y"`
6. Push tag: `git push origin v0.X.Y`
7. GitHub Actions will create release automatically

## Links

- [Repository](https://github.com/ssahani/netevd)
- [Issue Tracker](https://github.com/ssahani/netevd/issues)
- [Releases](https://github.com/ssahani/netevd/releases)
- [crates.io](https://crates.io/crates/netevd)
