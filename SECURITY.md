<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Security Policy

## Supported Versions

Security updates are provided for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

### How to Report

If you discover a security vulnerability in netevd, please report it responsibly:

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please email:
- **Email**: ssahani@redhat.com
- **Subject**: [SECURITY] netevd vulnerability report

### What to Include

Your report should include:

1. **Description**: Clear description of the vulnerability
2. **Impact**: What could an attacker achieve?
3. **Steps to Reproduce**: Detailed steps to reproduce the issue
4. **Proof of Concept**: Code or configuration demonstrating the issue (if applicable)
5. **Suggested Fix**: If you have ideas on how to fix it (optional)
6. **Your Information**: Name/handle for credit (optional)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Assessment**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: Within 7 days
  - High: Within 14 days
  - Medium: Within 30 days
  - Low: Next regular release

### Disclosure Policy

- We follow coordinated disclosure
- We will acknowledge your contribution (unless you prefer to remain anonymous)
- We will credit you in release notes and security advisories
- Please allow us time to fix before public disclosure
- We aim to release fixes before public disclosure

## Security Model

### Architecture Overview

netevd implements defense-in-depth security with multiple layers:

```
┌─────────────────────────────────────────────┐
│  Layer 5: System Hardening (systemd)        │
│  - NoNewPrivileges                          │
│  - ProtectSystem=strict                     │
│  - PrivateTmp                               │
└─────────────────────────────────────────────┘
┌─────────────────────────────────────────────┐
│  Layer 4: Execution Isolation               │
│  - Scripts run as netevd user               │
│  - No capability inheritance                │
│  - Validated environment only               │
└─────────────────────────────────────────────┘
┌─────────────────────────────────────────────┐
│  Layer 3: Input Validation                  │
│  - Interface name validation                │
│  - IP address sanitization                  │
│  - Shell metacharacter filtering            │
└─────────────────────────────────────────────┘
┌─────────────────────────────────────────────┐
│  Layer 2: Minimal Capabilities              │
│  - CAP_NET_ADMIN only                       │
│  - No capability inheritance                │
└─────────────────────────────────────────────┘
┌─────────────────────────────────────────────┐
│  Layer 1: Privilege Separation              │
│  - Starts as root (UID 0)                   │
│  - Drops to netevd user                     │
│  - Cannot regain privileges                 │
└─────────────────────────────────────────────┘
```

### Threat Model

#### In Scope

1. **Malicious Network Input**
   - Rogue DHCP servers
   - Crafted DBus messages
   - Malicious netlink messages

2. **Local Attacks**
   - Malicious scripts in script directories
   - Configuration file tampering
   - Resource exhaustion

3. **Privilege Escalation**
   - Attempts to regain root privileges
   - Capability leakage to child processes
   - Exploiting script execution

#### Out of Scope

1. **Physical Access**: Physical attacks on the machine
2. **Kernel Vulnerabilities**: Bugs in Linux kernel
3. **systemd Bugs**: Vulnerabilities in systemd itself
4. **Side Channels**: Timing attacks, cache attacks, etc.

### Security Guarantees

#### What netevd DOES protect against:

1. **Command Injection**
   - All environment variables are validated
   - Shell metacharacters are rejected
   - No shell is used for script execution

2. **Privilege Escalation**
   - Runs as unprivileged user
   - NoNewPrivileges prevents setuid
   - Minimal capabilities (CAP_NET_ADMIN only)

3. **Capability Leakage**
   - Child processes inherit no capabilities
   - Scripts run without network admin rights
   - Ambient capabilities not used for scripts

4. **Resource Exhaustion** (partial)
   - systemd can limit resources
   - Event processing is async (non-blocking)
   - Failed scripts don't crash daemon

#### What netevd DOES NOT protect against:

1. **Malicious Scripts**: If you install a malicious script in `/etc/netevd/*.d/`, it will run
2. **Configuration Tampering**: If attacker has write access to `/etc/netevd/`
3. **Root Compromise**: If attacker has root, they can modify the binary
4. **Kernel Exploits**: netevd cannot protect against kernel vulnerabilities

### Security Features

#### 1. Privilege Dropping

netevd starts as root (to acquire CAP_NET_ADMIN) but immediately drops privileges:

```rust
// Pseudocode
if running_as_root() {
    // Step 1: Enable capability retention
    prctl(PR_SET_KEEPCAPS, 1);

    // Step 2: Drop to netevd user
    setgid(netevd_gid);
    setuid(netevd_uid);

    // Step 3: Disable capability retention
    prctl(PR_SET_KEEPCAPS, 0);

    // Step 4: Set minimal capabilities
    clear_all_capabilities();
    set_capability(CAP_NET_ADMIN, PERMITTED);
    set_capability(CAP_NET_ADMIN, EFFECTIVE);
}
```

**Result**: Process runs as `netevd` user with only CAP_NET_ADMIN.

#### 2. Input Validation

All external input is validated before use:

```rust
// Interface names: only alphanumeric, -, _, .
validate_interface_name("eth0")      // ✅ Pass
validate_interface_name("eth0; rm")  // ❌ Reject

// IP addresses: strict parsing
validate_ip_address("192.168.1.1")   // ✅ Pass
validate_ip_address("$(whoami)")     // ❌ Reject

// Hostnames: RFC compliant only
validate_hostname("example.com")     // ✅ Pass
validate_hostname("$(id).com")       // ❌ Reject
```

**Validation Rules**:
- Interface names: `^[a-zA-Z0-9._-]+$`
- IP addresses: Parsed by `std::net::IpAddr`
- Hostnames: RFC 1123 compliance
- Environment values: No shell metacharacters (`;$`\`&|<>()`)

#### 3. Script Execution Safety

Scripts are executed safely:

1. **Direct Execution**: No shell intermediary
2. **Validated Environment**: Only safe variables passed
3. **No Capabilities**: Scripts inherit no special privileges
4. **Unprivileged User**: Scripts run as `netevd` user
5. **Error Isolation**: Script failure doesn't crash daemon

#### 4. systemd Hardening

The systemd service file includes hardening options:

```ini
[Service]
# Prevent privilege escalation
NoNewPrivileges=true

# Filesystem protection
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=/run/netevd

# Namespace isolation
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Network access required
PrivateNetwork=false

# Capabilities
AmbientCapabilities=CAP_NET_ADMIN
CapabilityBoundingSet=CAP_NET_ADMIN

# User
User=netevd
Group=netevd
```

## Known Security Considerations

### 1. Script Directory Permissions

**Issue**: If an attacker can write to `/etc/netevd/*.d/`, they can execute code.

**Mitigation**:
- Ensure proper permissions: `chmod 750 /etc/netevd`
- Owner should be root: `chown root:netevd /etc/netevd`
- Only root should be able to write scripts

**Verification**:
```bash
ls -la /etc/netevd
# Should show: drwxr-x--- root netevd
```

### 2. Configuration File Security

**Issue**: Configuration file contains sensitive information.

**Mitigation**:
- Proper permissions: `chmod 640 /etc/netevd/netevd.yaml`
- Owner root: `chown root:netevd /etc/netevd/netevd.yaml`

**Verification**:
```bash
ls -la /etc/netevd/netevd.yaml
# Should show: -rw-r----- root netevd
```

### 3. DHCP Security

**Issue**: Malicious DHCP server could send crafted responses.

**Mitigation**:
- All DHCP data is validated before use
- Shell metacharacters are rejected
- Use trusted networks only

**Best Practice**:
- Use DHCP snooping on switches
- Validate DHCP server authenticity
- Monitor for rogue DHCP servers

### 4. DBus Security

**Issue**: Malicious DBus messages could be sent.

**Mitigation**:
- DBus enforces access control
- Only listen to system bus
- Validate all DBus message contents

### 5. Netlink Security

**Issue**: Netlink messages come from kernel (trusted).

**Mitigation**:
- Netlink messages are from kernel only
- No user-space can inject netlink messages
- Still validate all data

## Security Best Practices

### For Users

1. **Keep Updated**
   ```bash
   # Check for updates regularly
   cargo install netevd --force
   ```

2. **Restrict Script Directory Access**
   ```bash
   sudo chown -R root:netevd /etc/netevd
   sudo chmod -R 750 /etc/netevd
   ```

3. **Review Scripts Before Installing**
   ```bash
   # Always review third-party scripts
   cat /path/to/script.sh
   # Only install if you trust it
   sudo cp /path/to/script.sh /etc/netevd/routable.d/
   ```

4. **Monitor Logs**
   ```bash
   # Watch for suspicious activity
   sudo journalctl -u netevd -f
   ```

5. **Use AppArmor/SELinux** (if available)
   - Additional layer of protection
   - Confines netevd further

### For Developers

1. **Never Trust Input**
   - Always validate external data
   - Use existing validation functions
   - Add tests for edge cases

2. **Avoid Shell Execution**
   - Use direct execution (`Command::new()`)
   - Never use `sh -c` with user input
   - Validate before passing to system

3. **Principle of Least Privilege**
   - Request minimal capabilities
   - Drop privileges early
   - Don't request more than needed

4. **Secure by Default**
   - Safe defaults in configuration
   - Opt-in for risky features
   - Clear security warnings

5. **Code Review**
   - All PRs require review
   - Security-sensitive code needs extra scrutiny
   - Use `cargo clippy` and `cargo audit`

## Security Audits

### Internal Audits

- Code review for all changes
- Security-focused testing
- Regular dependency updates
- Automated security scanning

### External Audits

We welcome external security audits. If you're interested in auditing netevd:

1. Contact us at ssahani@redhat.com
2. We can provide guidance on areas of focus
3. We appreciate responsible disclosure
4. We will credit auditors in our security acknowledgments

## Security Acknowledgments

We thank the following people for responsibly disclosing security issues:

*(None reported yet)*

## Security Tools

### Recommended Tools

1. **cargo-audit**: Check for vulnerable dependencies
   ```bash
   cargo install cargo-audit
   cargo audit
   ```

2. **cargo-deny**: Check licenses and security advisories
   ```bash
   cargo install cargo-deny
   cargo deny check
   ```

3. **clippy**: Rust linter with security checks
   ```bash
   cargo clippy -- -D warnings
   ```

4. **AppArmor/SELinux**: Additional confinement

### Continuous Monitoring

```bash
# Setup GitHub Dependabot (automated)
# Checks for vulnerable dependencies

# Regular audits
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Compliance

### Standards

netevd aims to comply with:

- OWASP Top 10 (where applicable)
- CWE (Common Weakness Enumeration)
- NIST Cybersecurity Framework

### Certifications

Currently no formal certifications. Open to pursuing certifications if needed by users.

## Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE](https://cwe.mitre.org/)
- [Rust Security](https://www.rust-lang.org/policies/security)
- [Linux Capabilities](https://man7.org/linux/man-pages/man7/capabilities.7.html)

## Contact

- **Security Issues**: ssahani@redhat.com
- **General Issues**: https://github.com/ssahani/netevd/issues
- **PGP Key**: (if needed, request via email)

## Updates to This Policy

This security policy may be updated from time to time. Check the git history for changes:

```bash
git log -- SECURITY.md
```

Last Updated: 2026-01-22
