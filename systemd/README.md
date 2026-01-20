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
  - Runs as `netevd` user
  - Requires `CAP_NET_ADMIN` capability
  - Includes security hardening (NoNewPrivileges, ProtectSystem, etc.)

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
