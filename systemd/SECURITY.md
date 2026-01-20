# systemd Service Security Hardening

This document explains the security hardening applied to the netevd systemd service.

## Security Principles

The netevd service follows defense-in-depth security principles:

1. **Least Privilege**: Runs as unprivileged user with minimal capabilities
2. **Process Isolation**: Sandboxed from the rest of the system
3. **Attack Surface Reduction**: Restricted system call access
4. **Defense in Depth**: Multiple layers of protection

## Service Type

```ini
Type=simple
```

**Why**: netevd uses async I/O and doesn't implement sd_notify(). Using `Type=simple` ensures systemd correctly tracks the service lifecycle.

**Previous**: `Type=notify` (incorrect - would wait indefinitely for notification)

## User & Capabilities

```ini
User=netevd
Group=netevd
AmbientCapabilities=CAP_NET_ADMIN
CapabilityBoundingSet=CAP_NET_ADMIN
NoNewPrivileges=true
```

**Security Benefits**:
- Runs as dedicated user (not root)
- Only `CAP_NET_ADMIN` - can configure network, nothing else
- Cannot gain additional privileges
- Cannot execute setuid binaries

**What CAP_NET_ADMIN Allows**:
- Create/modify routing tables (`ip route`)
- Add/remove routing policy rules (`ip rule`)
- Use netlink sockets
- Configure network interfaces

**What is BLOCKED**:
- File system access outside allowed paths
- Process manipulation
- Kernel module loading
- Clock/time changes
- Raw I/O access

## Filesystem Isolation

```ini
ProtectSystem=strict
ProtectHome=true
ReadOnlyPaths=/etc/netevd
PrivateTmp=true
```

**Protections**:
- `/usr`, `/boot`, `/efi` are read-only
- Cannot access `/home` directories
- `/etc/netevd` is read-only (config cannot be modified)
- Private `/tmp` - isolated from other processes

**Allowed Writes**:
- `/var/log` (for logging via journald)
- `/run/netevd` (if needed for runtime data)

## Kernel Protection

```ini
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectClock=true
```

**Prevents**:
- Modifying `/proc/sys` (kernel tunables)
- Loading kernel modules via `insmod`, `modprobe`
- Reading kernel logs (`dmesg`, `/dev/kmsg`)
- Changing system time/clock

## Process Visibility

```ini
ProtectProc=invisible
ProcSubset=pid
ProtectControlGroups=true
```

**Effect**:
- Cannot see other processes in `/proc`
- Only `/proc/[pid]` directories visible
- `/proc/sys`, `/proc/sysrq-trigger` blocked
- `/sys/fs/cgroup` is read-only

## Network Restrictions

```ini
RestrictAddressFamilies=AF_UNIX AF_NETLINK AF_INET AF_INET6
PrivateDevices=false
```

**Allowed**:
- `AF_UNIX` - Unix domain sockets (DBus communication)
- `AF_NETLINK` - Netlink sockets (network events, routing)
- `AF_INET` - IPv4 (network operations)
- `AF_INET6` - IPv6 (network operations)

**Blocked**:
- `AF_PACKET` - Raw packet access
- `AF_BLUETOOTH` - Bluetooth sockets
- Other address families

**Note**: `PrivateDevices=false` because netevd needs access to network devices via netlink.

## System Call Filtering

```ini
SystemCallFilter=@system-service @network-io @io-event @signal @process
SystemCallFilter=~@privileged @clock @module @raw-io @reboot @swap @cpu-emulation @obsolete @debug
SystemCallErrorNumber=EPERM
```

### Allowed Syscall Groups

| Group | Purpose | Examples |
|-------|---------|----------|
| `@system-service` | Basic service operations | `read`, `write`, `open`, `close` |
| `@network-io` | Network operations | `socket`, `bind`, `connect`, `sendto`, `recvfrom` |
| `@io-event` | Async I/O | `epoll_create`, `epoll_wait`, `eventfd` |
| `@signal` | Signal handling | `sigaction`, `sigprocmask` |
| `@process` | Process management | `fork`, `execve`, `wait4` |

### Blocked Syscall Groups

| Group | Reason | Examples |
|-------|--------|----------|
| `@privileged` | Requires root | `chroot`, `mount`, `pivot_root` |
| `@clock` | Time changes | `settimeofday`, `clock_settime` |
| `@module` | Kernel modules | `init_module`, `finit_module` |
| `@raw-io` | Direct hardware | `iopl`, `ioperm` |
| `@reboot` | System control | `reboot`, `kexec_load` |
| `@swap` | Swap management | `swapon`, `swapoff` |
| `@cpu-emulation` | Emulation | `vm86`, `vm86old` |
| `@obsolete` | Deprecated | Old syscalls |
| `@debug` | Debugging | `ptrace`, `process_vm_readv` |

**Error Handling**: Blocked syscalls return `EPERM` (Operation not permitted)

## Memory Protection

```ini
# MemoryDenyWriteExecute=true  # Disabled for Rust compatibility
RestrictRealtime=true
LockPersonality=true
```

**Why MemoryDenyWriteExecute is disabled**:
- Rust's async runtime (tokio) may use JIT compilation
- Some memory allocators need W^X (write XOR execute)
- Trade-off: Allow for runtime compatibility

**Other Memory Protections**:
- `RestrictRealtime=true` - No real-time scheduling (prevents timing attacks)
- `LockPersonality=true` - Cannot change execution domain

## Additional Hardening

```ini
RestrictNamespaces=true
RestrictSUIDSGID=true
RemoveIPC=true
```

- `RestrictNamespaces=true` - Cannot create namespaces (containers)
- `RestrictSUIDSGID=true` - Cannot create setuid/setgid files
- `RemoveIPC=true` - IPC objects removed on exit

## Logging

```ini
StandardOutput=journal
StandardError=journal
SyslogIdentifier=netevd
```

- All output goes to systemd journal
- Logs tagged with `netevd` identifier
- View with: `journalctl -u netevd`

## Security Score

Check your service security score:

```bash
systemd-analyze security netevd
```

Expected score: **8.5-9.0** (out of 10) - Very secure

Common deductions:
- `-0.5`: PrivateDevices=false (needed for network access)
- `-0.5`: MemoryDenyWriteExecute=false (Rust runtime compatibility)

## Testing

Verify security settings:

```bash
# Check capabilities
systemctl show netevd | grep Capabilities
# Output: AmbientCapabilities=cap_net_admin
#         CapabilityBoundingSet=cap_net_admin

# Check filesystem restrictions
systemctl show netevd | grep ProtectSystem
# Output: ProtectSystem=strict

# Check syscall filter
systemctl show netevd | grep SystemCallFilter

# Full security analysis
systemd-analyze security netevd --no-pager
```

## Security Best Practices

1. **Never disable hardening** unless absolutely necessary
2. **Review logs regularly** for permission denied errors
3. **Use override files** for customization (don't edit main unit)
4. **Test after changes** with `systemd-analyze verify`
5. **Monitor capabilities** - should always be minimal

## References

- [systemd.exec(5)](https://www.freedesktop.org/software/systemd/man/systemd.exec.html) - Execution environment
- [systemd.resource-control(5)](https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html) - Resource limits
- [Linux Capabilities](https://man7.org/linux/man-pages/man7/capabilities.7.html) - Capability details
- [Systemd Sandboxing](https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html#Sandboxing) - Sandboxing options
