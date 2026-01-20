# netevd Improvement Recommendations

## 🔴 Critical (Must Fix Before Production)

### 1. Integrate Generator-Based Listeners into Main Loop
**Status**: Listeners implemented but not started  
**Impact**: High - Core functionality not working  
**File**: `src/main.rs`

**Current Issue**:
```rust
// TODO: Add listener tasks (networkd or dhclient) here based on backend type
```

**Fix**:
```rust
// Add before tokio::select!:
let config_clone = config.clone();
let state_listener = state.clone();
let handle_listener = handle.clone();

// Then add to tokio::select!:
result = spawn_listener(config_clone, handle_listener, state_listener) => {
    warn!("Listener exited: {:?}", result);
}

// Helper function:
async fn spawn_listener(config: Config, handle: Handle, state: Arc<RwLock<NetworkState>>) -> Result<()> {
    match config.system.backend.as_str() {
        "systemd-networkd" => listeners::networkd::listen_networkd(config, handle, state).await,
        "NetworkManager" => listeners::networkmanager::listen_networkmanager(config, handle, state).await,
        "dhclient" => listeners::dhclient::watch_lease_file(config, handle, state).await,
        _ => anyhow::bail!("Unknown backend: {}", config.system.backend),
    }
}
```

### 2. Fix Deprecated rtnetlink API Usage
**Status**: 6 deprecation warnings  
**Impact**: Medium - Will break in future versions  
**Files**: `src/network/route.rs`, `src/network/routing_rule.rs`

**Fix**: Replace `.table(table as u8)` with `.table_id(table)`
```rust
// Before:
.table(table as u8)

// After:
.table_id(table)
```

### 3. Remove Routing Table u8 Limitation
**Status**: Current code limits table IDs to 0-255  
**Impact**: Medium - Limits scalability  
**Current**: ROUTE_TABLE_BASE = 200, ifindex limited to 55

**Fix**:
```rust
// In route.rs and routing_rule.rs:
.table_id(table)  // Accepts u32, not u8

// This allows table IDs up to 2^32-1
```

---

## 🟡 High Priority (Performance & Stability)

### 4. Replace Polling with Real-Time Netlink Subscriptions
**Status**: Currently polls every 5 seconds  
**Impact**: High - Latency and efficiency  
**File**: `src/network/watcher.rs`

**Current**:
```rust
const POLL_INTERVAL: Duration = Duration::from_secs(5);
// Uses periodic polling
```

**Improvement**: Implement proper netlink multicast groups
```rust
// Use netlink-sys multicast subscriptions:
const RTMGRP_LINK: u32 = 0x1;
const RTMGRP_IPV4_IFADDR: u32 = 0x10;
const RTMGRP_IPV4_ROUTE: u32 = 0x40;

// Subscribe to real-time events with <100ms latency
```

**Benefits**:
- Reduces latency from 5s to <100ms
- Reduces CPU usage (no polling)
- Immediate response to network changes

### 5. Add IPv6 Support for Routing Policy Rules
**Status**: Only IPv4 currently supported  
**Impact**: Medium - Missing feature for IPv6 networks  
**Files**: `src/network/watcher.rs`, `src/network/route.rs`

**Fix**:
```rust
// In watch_addresses():
let addresses = get_all_addresses(handle, ifindex).await?;  // Already supports IPv6
// Just ensure configure_network() handles IPv6 addresses
```

### 6. Add Graceful Shutdown with Cleanup
**Status**: No cleanup on shutdown  
**Impact**: Medium - Leaves routing rules/routes on exit  
**File**: `src/main.rs`

**Fix**:
```rust
// Before shutdown, cleanup:
async fn cleanup_on_shutdown(state: Arc<RwLock<NetworkState>>, handle: Handle) -> Result<()> {
    info!("Cleaning up routing configuration...");
    let state_read = state.read().await;
    
    // Remove all custom routing rules and routes
    for (ifindex, table) in &state_read.routes {
        remove_route(&handle, ifindex.0, table.table).await?;
    }
    
    for (address, rule) in &state_read.routing_rules_from {
        remove_routing_rules(&handle, *address, rule.table).await?;
    }
    
    info!("Cleanup complete");
    Ok(())
}
```

---

## 🟢 Medium Priority (Code Quality)

### 7. Fix 71 Compiler Warnings
**Status**: Unused imports, variables, constants  
**Impact**: Low - Code cleanliness  

**Fix**: Run `cargo fix` or manually clean up:
```bash
cargo fix --bin netevd
cargo clippy --fix
```

### 8. Add Comprehensive Error Context
**Status**: Some errors lack context  
**Impact**: Medium - Debugging difficulty  

**Example**:
```rust
// Before:
.await?;

// After:
.await
.with_context(|| format!("Failed to configure routing for interface {}", ifindex))?;
```

### 9. Add Configuration Validation
**Status**: No validation on startup  
**Impact**: Medium - Runtime failures  
**File**: `src/config/mod.rs`

**Fix**:
```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        // Validate backend
        match self.system.backend.as_str() {
            "systemd-networkd" | "NetworkManager" | "dhclient" => {},
            other => anyhow::bail!("Invalid backend: {}", other),
        }
        
        // Validate log level
        match self.system.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            other => anyhow::bail!("Invalid log level: {}", other),
        }
        
        Ok(())
    }
}
```

### 10. Add Structured Logging with Context
**Status**: Basic logging only  
**Impact**: Medium - Observability  

**Fix**:
```rust
use tracing::{info, warn, instrument};

#[instrument(skip(handle, state))]
async fn configure_network(...) -> Result<()> {
    // Automatic span with function args logged
}
```

---

## 🔵 Nice to Have (Features & Enhancements)

### 11. Add Health Check Endpoint
**Purpose**: Monitoring & orchestration  
**Implementation**:
```rust
// Add HTTP health endpoint on localhost:9090
use axum::{Router, routing::get};

async fn health_check() -> &'static str { "OK" }

let app = Router::new().route("/health", get(health_check));
tokio::spawn(async {
    axum::Server::bind(&"127.0.0.1:9090".parse().unwrap())
        .serve(app.into_make_service())
        .await
});
```

### 12. Add Metrics Export (Prometheus Format)
**Purpose**: Monitoring & alerting  
**Metrics to Track**:
- Active network interfaces
- Routing rules configured
- Script executions (success/failure)
- DBus signal events received
- Address/route change events

### 13. Add Configuration Hot-Reload
**Purpose**: Update config without restart  
**Implementation**: Watch config file with `notify` crate

### 14. Add CLI Status Command
**Purpose**: Query daemon state  
**Implementation**:
```bash
netevd status              # Show overall status
netevd list-interfaces     # Show monitored interfaces
netevd list-rules          # Show active routing rules
netevd reload              # Reload configuration
```

### 15. Add Integration Tests
**Purpose**: Ensure functionality  
**Files**: Create `tests/` directory

```rust
#[tokio::test]
async fn test_routing_policy_rules() {
    // Test routing rule creation
}

#[tokio::test]
async fn test_script_execution() {
    // Test script execution with mock
}
```

### 16. Add Example Scripts
**Purpose**: User documentation  
**Directory**: `examples/scripts/`

```bash
examples/scripts/
├── routable.d/
│   └── 01-notify-slack.sh
├── activated.d/
│   └── 01-update-dns.sh
└── disconnected.d/
    └── 01-cleanup.sh
```

### 17. Add systemd Socket Activation
**Purpose**: On-demand startup  
**File**: `systemd/ and examples/netevd.socket`

### 18. Add Rate Limiting for Script Execution
**Purpose**: Prevent script flooding  
**Implementation**:
```rust
use tokio::time::{interval, Duration};

// Max 10 script executions per minute per interface
```

### 19. Add Support for Custom Routing Tables Names
**Purpose**: Better readability  
**File**: `/etc/iproute2/rt_tables`

```rust
// Auto-generate entries like:
// 202    netevd_eth0
// 203    netevd_eth1
```

### 20. Add DBus Service for External Control
**Purpose**: Allow other apps to query/control netevd  
**Service**: `org.freedesktop.netevd`

---

## 📊 Performance Optimizations

### 21. Connection Pooling for DBus
**Current**: Creates new connection per operation  
**Fix**: Reuse connection

### 22. Batch Routing Rule Updates
**Current**: One rule at a time  
**Fix**: Batch multiple rules in single netlink message

### 23. Add LRU Cache for Link State File Parsing
**Current**: Parses INI file every time  
**Fix**: Cache parsed state with TTL

---

## 🔒 Security Enhancements

### 24. Verify Capabilities After Drop
**Purpose**: Ensure security worked  
```rust
// After drop_privileges(), verify:
let caps = caps::read(None, CapSet::Effective)?;
assert!(caps.contains(&Capability::CAP_NET_ADMIN));
```

### 25. Add SELinux Policy
**Purpose**: Confine daemon  
**File**: `systemd/ and examples/netevd.te`

### 26. Validate Script Paths
**Purpose**: Prevent directory traversal  
**Fix**: Ensure scripts are within allowed directories

---

## 📝 Documentation Improvements

### 27. Add Architecture Diagram
### 28. Add User Guide with Examples
### 29. Add Troubleshooting Section
### 30. Add Migration Guide (from network-broker)

---

## Priority Implementation Order

1. **Week 1**: Critical items #1-3 (Get it working!)
2. **Week 2**: High priority #4-6 (Performance & stability)
3. **Week 3**: Medium priority #7-10 (Code quality)
4. **Week 4**: Nice to have #11-16 (Features)
5. **Week 5**: Documentation & polish #27-30

---

## Quick Wins (Can Do Now)

```bash
# Fix deprecation warnings:
cargo fix --bin netevd

# Fix clippy warnings:
cargo clippy --fix

# Add missing documentation:
cargo doc --open

# Format code:
cargo fmt
```
