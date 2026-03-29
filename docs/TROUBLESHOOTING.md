<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Troubleshooting

## Quick Diagnosis

```bash
sudo systemctl status netevd                    # Is it running?
sudo journalctl -u netevd -n 50 --no-pager      # Recent logs
sudo journalctl -u netevd | grep -i error        # Errors only
```

## Service Won't Start

**Check logs first:**
```bash
sudo systemctl status netevd -l
sudo journalctl -u netevd -n 50 --no-pager
```

**User doesn't exist:**
```bash
# Error: "User netevd could not be found"
sudo useradd -r -M -s /usr/bin/nologin -d /nonexistent netevd
sudo systemctl restart netevd
```

**Config syntax error:**
```bash
# Validate YAML
yamllint /etc/netevd/netevd.yaml
python3 -c "import yaml; yaml.safe_load(open('/etc/netevd/netevd.yaml'))"
```

**Permission denied:**
```bash
sudo chmod 644 /etc/netevd/netevd.yaml
sudo chown root:netevd /etc/netevd/netevd.yaml
sudo chmod 755 /etc/netevd
```

**Missing capabilities:**
```bash
sudo systemctl cat netevd | grep AmbientCapabilities
# Should show: AmbientCapabilities=CAP_NET_ADMIN
sudo systemctl daemon-reload && sudo systemctl restart netevd
```

## No Events Received

**Check your backend is running:**
```bash
# systemd-networkd
sudo systemctl status systemd-networkd
busctl tree org.freedesktop.network1

# NetworkManager
sudo systemctl status NetworkManager
busctl tree org.freedesktop.NetworkManager

# dhclient
ps aux | grep dhclient
ls -la /var/lib/dhcp/dhclient.leases
```

**Trigger a test event:**
```bash
sudo ip link set eth0 down && sudo ip link set eth0 up
sudo journalctl -u netevd -n 20
```

**Check interface is monitored:**
```bash
grep -A10 "^monitoring:" /etc/netevd/netevd.yaml
# If specific interfaces listed, make sure yours is included
# Empty list = monitor all
```

## Scripts Not Executing

**Check permissions:**
```bash
ls -la /etc/netevd/routable.d/
# Must show 'x' in permissions: -rwxr-xr-x
sudo chmod +x /etc/netevd/routable.d/*.sh
```

**Check shebang:**
```bash
head -1 /etc/netevd/routable.d/01-test.sh
# Must be: #!/bin/bash
```

**Check syntax:**
```bash
bash -n /etc/netevd/routable.d/01-test.sh
```

**Test manually:**
```bash
sudo env LINK=eth0 LINKINDEX=2 STATE=routable \
    BACKEND=systemd-networkd ADDRESSES="192.168.1.100" \
    bash -x /etc/netevd/routable.d/01-test.sh
```

**Check logs for execution attempts:**
```bash
sudo journalctl -u netevd | grep -i "executing\|script"
```

## Routing Rules Not Created

**Is the interface configured for policy rules?**
```bash
grep -A5 "^routing:" /etc/netevd/netevd.yaml
# Interface must be listed under policy_rules
```

**Is the interface routable?**
```bash
ip addr show eth1
# Must have an IP address and be UP
```

**Check current rules:**
```bash
ip rule list
ip route show table 203    # table = 200 + ifindex
```

**Check capabilities:**
```bash
sudo systemctl show netevd | grep Capabilit
# Must include CAP_NET_ADMIN
```

**Manual cleanup if needed:**
```bash
sudo ip rule del from 192.168.1.100 table 203
sudo ip rule del to 192.168.1.100 table 203
sudo ip route flush table 203
sudo systemctl restart netevd
```

## Traffic Using Wrong Interface

```bash
# Check rules exist
ip rule list | grep 192.168.1.100

# Check custom table has a route
ip route show table 203

# Test routing decision
ip route get 8.8.8.8 from 192.168.1.100
# Should show: dev eth1 table 203

# If wrong, flush cache and restart
ip route flush cache
sudo systemctl restart netevd
```

## High CPU Usage

```bash
# Check event rate
sudo journalctl -u netevd --since "1 minute ago" | wc -l

# Common cause: interface flapping (check physical connection/driver)
sudo journalctl -u netevd --since "1 minute ago" | head -50
```

## Debug Logging

**Via config:**
```yaml
system:
  log_level: "debug"    # or "trace" for maximum detail
```

**Via environment:**
```bash
sudo systemctl edit netevd
# Add:
# [Service]
# Environment="RUST_LOG=debug"

sudo systemctl daemon-reload && sudo systemctl restart netevd
```

**Per-module tracing:**
```bash
# Only network module
RUST_LOG=netevd::network=trace

# Multiple modules
RUST_LOG=netevd::network=trace,netevd::listeners=debug
```

## Collect Diagnostic Report

```bash
#!/bin/bash
echo "=== System ===" && uname -a && cat /etc/os-release
echo "=== Version ===" && netevd --version
echo "=== Status ===" && systemctl status netevd --no-pager
echo "=== Config ===" && cat /etc/netevd/netevd.yaml
echo "=== Logs ===" && journalctl -u netevd -n 100 --no-pager
echo "=== Network ===" && ip link show && ip addr show && ip route show && ip rule list
echo "=== Scripts ===" && ls -laR /etc/netevd/*.d/
echo "=== Backend ===" && systemctl status systemd-networkd NetworkManager --no-pager 2>/dev/null
```

Save and attach to bug reports at https://github.com/ssahani/netevd/issues.

## See Also

- [Configuration](../CONFIGURATION.md) -- fix config issues
- [Installation](../INSTALL.md) -- setup problems
- [Architecture](ARCHITECTURE.md) -- understand internals
