<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Architecture

Technical overview of netevd's internals for contributors and curious users.

## System Overview

```
+-------------------------------------------------------------------+
|  External Systems                                                  |
|  +---------------+  +-----------------+  +-------------------+    |
|  | Linux Kernel   |  | systemd-networkd|  | NetworkManager    |    |
|  | (Netlink)      |  | (DBus)          |  | (DBus) / dhclient |    |
|  +-------+-------+  +--------+--------+  +---------+---------+    |
+----------|-------------------|----------------------|-------------+
           |                   |                      |
+----------|-------------------|----------------------|-------------+
|  netevd  |                   |                      |              |
|          v                   v                      v              |
|  +-------------+    +---------------+    +----------------+        |
|  | Netlink     |    | Backend       |    | File Watcher   |        |
|  | Watchers    |    | Listener      |    | (dhclient)     |        |
|  | (addr/route |    | (DBus sigs)   |    | (inotify)      |        |
|  |  /link)     |    |               |    |                |        |
|  +------+------+    +-------+-------+    +--------+-------+        |
|         |                   |                      |               |
|         +-------------------+----------------------+               |
|                             |                                      |
|                    +--------v---------+                             |
|                    |  NetworkState    |                             |
|                    |  Arc<RwLock<T>>  |                             |
|                    +--------+---------+                             |
|                             |                                      |
|              +--------------+--------------+                       |
|              |              |              |                       |
|        +-----v-----+  +----v----+  +------v------+                |
|        |  Routing   |  | Script  |  |  DBus calls |                |
|        |  manager   |  | exec    |  |  (resolved/ |                |
|        |            |  |         |  |   hostnamed)|                |
|        +------------+  +---------+  +-------------+                |
+-------------------------------------------------------------------+
```

## Concurrency Model

netevd uses a multi-threaded Tokio runtime. All event sources run as concurrent async tasks joined by `tokio::select!`:

```rust
tokio::select! {
    _ = watch_addresses(state.clone(), config.clone()) => {}
    _ = watch_routes(state.clone(), config.clone()) => {}
    _ = watch_links(state.clone(), config.clone()) => {}
    _ = spawn_listener(state.clone(), config.clone()) => {}
    _ = api_server(state.clone(), config.clone()) => {}
    _ = signal_handler() => {}  // SIGTERM/SIGINT
}
```

If any task exits, the daemon shuts down. The signal handler enables graceful shutdown.

### Shared State

All tasks share a single `NetworkState` via `Arc<RwLock<T>>`:

- **Read locks** (shared): API queries, state comparisons
- **Write locks** (exclusive): state mutations after events

Lock discipline: hold locks for the minimum duration, compute actions outside the lock, then execute side effects (routing, scripts) after releasing.

```rust
pub struct NetworkState {
    links: HashMap<String, u32>,              // name -> index
    link_names: HashMap<u32, String>,         // index -> name
    routes: HashMap<(u32, u32), Vec<Route>>,  // (ifindex, table) -> routes
    rules_from: HashMap<IpAddr, RoutingRule>, // source-based rules
    rules_to: HashMap<IpAddr, RoutingRule>,   // dest-based rules
    link_states: HashMap<u32, LinkState>,     // per-interface state
}
```

## Event Processing Pipeline

Every event flows through five stages:

1. **Reception** -- Netlink socket recv, DBus signal delivery, or inotify file event
2. **Parsing** -- Deserialize message, extract interface index, addresses, state
3. **Validation** -- Check if interface is monitored, validate data, apply filters
4. **State update** -- Acquire write lock, compare with current state, update if changed
5. **Actions** -- Configure routing rules, execute scripts, make DBus calls

Events that don't change state are dropped at stage 4 (no duplicate processing).

### Event Types

| Netlink Message | Handler | Actions |
|----------------|---------|---------|
| `RTM_NEWADDR` | `handle_address_new` | Update state, create routing rules, run scripts |
| `RTM_DELADDR` | `handle_address_del` | Update state, remove routing rules |
| `RTM_NEWLINK` | `handle_link_change` | Refresh link state |
| `RTM_DELLINK` | `handle_link_del` | Remove link from state |
| `RTM_NEWROUTE` | `handle_route_new` | Update routes, run `routes.d/` scripts |
| `RTM_DELROUTE` | `handle_route_del` | Update routes |

## Backend Implementations

### systemd-networkd

Listens for `PropertiesChanged` signals on `/org/freedesktop/network1/link/*` via DBus. On signal:

1. Extract link index from object path
2. In parallel: read state file from `/run/systemd/netif/links/N` and query netlink for addresses/routes
3. Merge data, build JSON payload
4. Update state, execute `routable.d/` (or `configured.d/`, `degraded.d/`) scripts

### NetworkManager

Listens for `StateChanged` signals on device objects via DBus. Maps NM device states to event directories (`activated.d/`, `disconnected.d/`). Queries device properties for interface name and details.

### dhclient

Uses inotify to watch `/var/lib/dhcp/dhclient.leases`. On file change:

1. Read and parse the lease file (custom INI-like format)
2. Extract IP, gateway, DNS, domain, hostname
3. Update state, execute scripts
4. Optionally push DNS to systemd-resolved and hostname to systemd-hostnamed via DBus

## Routing Policy Rules

When an interface listed in `routing.policy_rules` becomes routable:

```
Interface routable?
      |
      v
Configured for policy rules? --No--> Skip
      |
     Yes
      v
Get IP addresses and discover gateway
      |
      v
Calculate table ID (200 + ifindex)
      |
      v
Create rules:
  - from <ip> lookup <table>
  - to <ip> lookup <table>
      |
      v
Add default route via <gateway> dev <iface> table <table>
```

When an address is removed, the corresponding rules and routes are cleaned up automatically.

## Security Architecture

### Privilege Drop Sequence

```
1. Process starts as root (UID=0)
2. prctl(PR_SET_KEEPCAPS, 1)      -- capabilities survive setuid
3. setgid(netevd), setuid(netevd) -- drop to unprivileged user
4. prctl(PR_SET_KEEPCAPS, 0)      -- no further capability retention
5. capset: clear all, set CAP_NET_ADMIN (permitted + effective)
6. Fork for script execution       -- child inherits NO capabilities
```

### Input Validation

Three-layer validation for all external data:

1. **Type validation** -- interface names match `^[a-zA-Z0-9._-]+$`, IPs parsed by `std::net::IpAddr`, hostnames RFC 1123
2. **Injection check** -- reject shell metacharacters (`;$\`&|<>()`)
3. **Length check** -- enforce maximum lengths

Validation functions live in `src/system/validation.rs` and are used by the routing, script execution, and DBus modules.

## Module Dependencies

```
main.rs
  +-- config/mod.rs          (YAML parsing)
  +-- system/user.rs         (privilege drop)
  +-- system/capability.rs   (CAP_NET_ADMIN)
  +-- network/watcher.rs     (netlink event loops)
  |     +-- network/mod.rs   (NetworkState)
  |     +-- network/address.rs, link.rs, route.rs, routing_rule.rs
  +-- listeners/networkd/    (systemd-networkd backend)
  |     +-- system/execute.rs (script execution)
  |     +-- bus/resolved.rs, hostnamed.rs (DBus)
  +-- listeners/networkmanager/ (NM backend)
  +-- listeners/dhclient/    (dhclient backend)
  +-- api/server.rs          (REST API)
```

## Performance Characteristics

| Operation | Typical Latency |
|-----------|----------------|
| Netlink recv | < 1ms |
| Message parsing | 1-3ms |
| State update (write lock) | 2-5ms |
| Route configuration | 10-30ms |
| Script fork+exec | 30-100ms |
| **Total event-to-script** | **< 100ms** |

Memory: 3-5 MB RSS idle, 5-8 MB during event bursts. CPU: < 1% idle, 2-5% during events.

### Key Optimizations

- **Async I/O** -- non-blocking netlink sockets, async DBus calls
- **Lock minimization** -- prefer read locks, hold write locks briefly
- **O(1) lookups** -- HashMap for link name/index mappings
- **Lazy evaluation** -- skip processing for unmonitored interfaces
- **Zero-copy where possible** -- Arc for shared ownership

## Build

```
Source (*.rs) -> rustc (--release, LTO) -> linker -> netevd binary (~2 MB)

Key dependencies:
  tokio 1.35       -- async runtime
  rtnetlink 0.20   -- netlink operations
  zbus 5.13        -- async DBus
  axum 0.8         -- HTTP API
  serde/serde_yaml -- configuration
  nix 0.31         -- Unix syscalls
  caps 0.5         -- Linux capabilities
```

## See Also

- [Configuration](../CONFIGURATION.md) -- config options that affect architecture
- [API](API.md) -- REST API built on Axum
- [Security](../SECURITY.md) -- detailed security model
