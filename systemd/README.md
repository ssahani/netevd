# systemd Unit Files

This directory contains systemd service unit files for netevd.

## Installation

```bash
sudo install -Dm644 netevd.service /lib/systemd/system/netevd.service
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
```

## Files

- **netevd.service** - Main systemd service unit file

## Service Configuration

### User & Security
- Runs as unprivileged `netevd` user
- Requires only `CAP_NET_ADMIN` capability (network configuration)
- `NoNewPrivileges=true` - Cannot gain additional privileges
- `ReadOnlyPaths=/etc/netevd` - Configuration is read-only

### Process Isolation
- `PrivateTmp=true` - Private /tmp directory
- `ProtectHome=true` - No access to /home directories
- `ProtectSystem=strict` - Read-only root filesystem
- `ProtectKernelTunables=true` - Cannot modify kernel parameters
- `ProtectKernelModules=true` - Cannot load kernel modules
- `ProtectKernelLogs=true` - Cannot read kernel logs
- `ProtectClock=true` - Cannot modify system clock
- `ProtectControlGroups=true` - Read-only cgroups
- `ProtectProc=invisible` - Limited /proc visibility
- `ProcSubset=pid` - Only see own processes

### Network & System Calls
- `RestrictAddressFamilies=AF_UNIX AF_NETLINK AF_INET AF_INET6`
  - Only allowed network families (Unix sockets, Netlink, IPv4/IPv6)
- `SystemCallFilter=@system-service @network-io @io-event @signal @process`
  - Whitelist essential syscalls for network daemon
- Blocks: privileged operations, clock changes, kernel modules, raw I/O, reboot, swap

### Dependencies
- `After=systemd-networkd.service` - Start after networkd if present
- `After=NetworkManager.service` - Start after NetworkManager if present
- `Before=network.target` - Part of early network setup
- `Wants=network.target` - Requires basic network

## Customization

To customize the service configuration:

```bash
sudo systemctl edit netevd
```

This creates an override file at `/etc/systemd/system/netevd.service.d/override.conf`.

### Common Customizations

**Enable debug logging:**
```ini
[Service]
Environment="RUST_LOG=debug"
```

**Change configuration file path:**
```ini
[Service]
ExecStart=
ExecStart=/usr/bin/netevd --config /custom/path/netevd.yaml
```

**Add resource limits:**
```ini
[Service]
MemoryMax=50M
CPUQuota=50%
```

## Service Management

```bash
# Start service
sudo systemctl start netevd

# Stop service
sudo systemctl stop netevd

# Restart service
sudo systemctl restart netevd

# Check status
sudo systemctl status netevd

# View logs
sudo journalctl -u netevd -f
```
