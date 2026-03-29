<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Security Policy

## Reporting Vulnerabilities

**Do not** open public issues for security vulnerabilities.

Email **ssahani@redhat.com** with subject `[SECURITY] netevd vulnerability report`. Include:

1. Description and impact
2. Steps to reproduce
3. Proof of concept (if applicable)
4. Suggested fix (optional)

**Response timeline:** initial response within 48 hours, assessment within 7 days. Critical fixes ship within 7 days, high within 14 days, medium within 30 days.

We follow coordinated disclosure and will credit reporters in release notes.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.2.x | Yes |
| 0.1.x | Yes |
| < 0.1 | No |

## Security Model

netevd uses defense-in-depth with five layers:

### Layer 1: Privilege Separation

Starts as root to acquire `CAP_NET_ADMIN`, then immediately drops to the `netevd` user:

```
UID=0 -> prctl(PR_SET_KEEPCAPS, 1)
      -> setgid(netevd), setuid(netevd)
      -> prctl(PR_SET_KEEPCAPS, 0)
      -> capset(CAP_NET_ADMIN only)
```

The process cannot regain root.

### Layer 2: Minimal Capabilities

Only `CAP_NET_ADMIN` is retained (needed for routing table configuration). All other capabilities are cleared. Child processes (scripts) inherit no capabilities.

### Layer 3: Input Validation

All external data is validated before use:

- **Interface names:** `^[a-zA-Z0-9._-]+$` only
- **IP addresses:** parsed by `std::net::IpAddr` (strict)
- **Hostnames:** RFC 1123 compliant
- **Environment values:** shell metacharacters (`;$\`&|<>()`) are rejected

### Layer 4: Execution Isolation

Scripts are executed directly (`Command::new()`), never through a shell. They run as the `netevd` user with no capabilities and receive only validated environment variables.

### Layer 5: systemd Hardening

```ini
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
AmbientCapabilities=CAP_NET_ADMIN
CapabilityBoundingSet=CAP_NET_ADMIN
```

## Threat Model

### In scope

| Threat | Mitigation |
|--------|------------|
| Malicious DHCP server | Input validation rejects shell metacharacters |
| Command injection via environment | Variables sanitized; direct exec, no shell |
| Privilege escalation | `netevd` user, `NoNewPrivileges`, minimal capabilities |
| Capability leakage to scripts | Child processes inherit no capabilities |
| Filesystem tampering | `ProtectSystem=strict`, read-only root |

### Out of scope

- Physical access to the machine
- Kernel vulnerabilities
- Bugs in systemd or DBus themselves
- Timing / side-channel attacks

### What netevd does NOT protect against

- **Malicious scripts** placed in `/etc/netevd/*.d/` -- if an attacker has write access to those directories, they can execute code
- **Root compromise** -- a root attacker can replace the binary
- **Configuration tampering** -- requires proper file permissions

## Hardening Checklist

```bash
# Script directories: only root can write
sudo chown -R root:netevd /etc/netevd
sudo chmod -R 750 /etc/netevd

# Config file: readable by netevd, writable by root only
sudo chmod 640 /etc/netevd/netevd.yaml

# Verify capabilities
sudo systemctl show netevd | grep Capabilit

# Audit dependencies
cargo audit
```

## Security Tools

```bash
cargo install cargo-audit && cargo audit     # Vulnerable dependencies
cargo install cargo-deny && cargo deny check  # License and advisory checks
cargo clippy -- -D warnings                   # Lint with security checks
```

## Contact

- **Security issues:** ssahani@redhat.com
- **General issues:** https://github.com/ssahani/netevd/issues
