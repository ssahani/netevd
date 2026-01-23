<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Architecture Deep Dive

This document provides a comprehensive technical overview of netevd's internal architecture.

## Table of Contents

- [System Architecture](#system-architecture)
- [Concurrency Model](#concurrency-model)
- [State Management](#state-management)
- [Event Processing Pipeline](#event-processing-pipeline)
- [Backend Implementations](#backend-implementations)
- [Network Operations](#network-operations)
- [Security Architecture](#security-architecture)
- [Performance Optimizations](#performance-optimizations)

## System Architecture

### High-Level Architecture

```mermaid
graph TB
    subgraph "External Systems"
        KERNEL[Linux Kernel<br/>Netlink Sockets]
        NETWORKD[systemd-networkd<br/>DBus]
        NM[NetworkManager<br/>DBus]
        DHCP[dhclient<br/>Lease Files]
    end

    subgraph "netevd Core"
        MAIN[main.rs<br/>Tokio Runtime]

        subgraph "Async Tasks"
            ADDR[Address Watcher]
            ROUTE[Route Watcher]
            LINK[Link Watcher]
            LISTEN[Backend Listener]
        end

        subgraph "Shared State"
            STATE[(NetworkState<br/>Arc RwLock)]
        end

        subgraph "Actions"
            EXEC[Script Executor]
            ROUTING[Routing Manager]
            DBUS[DBus Client]
        end
    end

    subgraph "External Actions"
        SCRIPTS[User Scripts<br/>/etc/netevd/*.d/]
        TABLES[Routing Tables]
        RESOLVED[systemd-resolved]
    end

    KERNEL -.->|Netlink Events| ADDR
    KERNEL -.->|Netlink Events| ROUTE
    KERNEL -.->|Netlink Events| LINK

    NETWORKD -.->|DBus Signals| LISTEN
    NM -.->|DBus Signals| LISTEN
    DHCP -.->|File Changes| LISTEN

    ADDR --> STATE
    ROUTE --> STATE
    LINK --> STATE
    LISTEN --> STATE

    STATE --> EXEC
    STATE --> ROUTING
    STATE --> DBUS

    EXEC --> SCRIPTS
    ROUTING --> TABLES
    DBUS --> RESOLVED

    MAIN -.-> ADDR
    MAIN -.-> ROUTE
    MAIN -.-> LINK
    MAIN -.-> LISTEN

    style MAIN fill:#2196f3,color:#fff
    style STATE fill:#ffeb3b
    style KERNEL fill:#9c27b0,color:#fff
```

### Component Layers

```mermaid
graph LR
    subgraph "Layer 1: Input"
        L1A[Netlink<br/>Multicast]
        L1B[DBus<br/>Signals]
        L1C[File<br/>Watchers]
    end

    subgraph "Layer 2: Event Processing"
        L2A[Parse]
        L2B[Validate]
        L2C[Filter]
    end

    subgraph "Layer 3: State Management"
        L3A[Read State]
        L3B[Compute Diff]
        L3C[Update State]
    end

    subgraph "Layer 4: Action Dispatch"
        L4A[Routing Logic]
        L4B[Script Selector]
        L4C[DBus Calls]
    end

    subgraph "Layer 5: Execution"
        L5A[Apply Routes]
        L5B[Run Scripts]
        L5C[Update Services]
    end

    L1A & L1B & L1C --> L2A
    L2A --> L2B --> L2C
    L2C --> L3A --> L3B --> L3C
    L3C --> L4A & L4B & L4C
    L4A --> L5A
    L4B --> L5B
    L4C --> L5C

    style L3B fill:#ffeb3b
    style L4B fill:#4caf50,color:#fff
```

## Concurrency Model

### Tokio Runtime Architecture

```mermaid
graph TB
    subgraph "Tokio Runtime (Multi-threaded)"
        SCHED[Task Scheduler]

        subgraph "Async Tasks"
            T1[Address Watcher<br/>Infinite Loop]
            T2[Route Watcher<br/>Infinite Loop]
            T3[Link Watcher<br/>Infinite Loop]
            T4[Backend Listener<br/>Event Driven]
            T5[Signal Handler<br/>SIGTERM/SIGINT]
            T6[API Server<br/>HTTP Listener]
        end
    end

    subgraph "Shared Resources"
        STATE[(NetworkState<br/>Arc RwLock T)]
        CFG[(Config<br/>Arc RwLock T)]
        METRICS[(Metrics<br/>Arc Mutex T)]
    end

    SCHED -.->|spawns| T1
    SCHED -.->|spawns| T2
    SCHED -.->|spawns| T3
    SCHED -.->|spawns| T4
    SCHED -.->|spawns| T5
    SCHED -.->|spawns| T6

    T1 & T2 & T3 & T4 -->|write lock| STATE
    T6 -->|read lock| STATE

    T1 & T2 & T3 & T4 -->|read lock| CFG

    T1 & T2 & T3 & T4 & T6 -->|increment| METRICS

    style SCHED fill:#2196f3,color:#fff
    style STATE fill:#ff9800,color:#fff
```

### Event Loop Structure

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize shared state
    let state = Arc::new(RwLock::new(NetworkState::new()));
    let config = Arc::new(RwLock::new(load_config()?));

    // Spawn concurrent tasks
    tokio::select! {
        // Netlink watchers (high priority)
        _ = watch_addresses(state.clone(), config.clone()) => {
            error!("Address watcher stopped");
        }
        _ = watch_routes(state.clone(), config.clone()) => {
            error!("Route watcher stopped");
        }
        _ = watch_links(state.clone(), config.clone()) => {
            error!("Link watcher stopped");
        }

        // Backend listener (event-driven)
        _ = spawn_listener(state.clone(), config.clone()) => {
            error!("Backend listener stopped");
        }

        // API server (if enabled)
        _ = api_server(state.clone(), config.clone()) => {
            error!("API server stopped");
        }

        // Signal handler (graceful shutdown)
        _ = signal_handler() => {
            info!("Received shutdown signal");
        }
    }

    Ok(())
}
```

### Lock Strategy

```mermaid
sequenceDiagram
    participant T1 as Address Watcher
    participant T2 as Route Watcher
    participant T3 as API Handler
    participant S as NetworkState

    Note over T1,S: Write-heavy operations
    T1->>S: Write Lock (exclusive)
    S-->>T1: Locked
    T1->>T1: Modify state
    T1->>S: Unlock

    Note over T2,S: Write-heavy operations
    T2->>S: Write Lock (exclusive)
    Note over T1,T2: T1 waits if T2 has lock
    S-->>T2: Locked
    T2->>T2: Modify state
    T2->>S: Unlock

    Note over T3,S: Read-only operations
    T3->>S: Read Lock (shared)
    S-->>T3: Locked

    Note over T1,T3: Multiple readers OK
    T1->>S: Read Lock (shared)
    S-->>T1: Locked

    T3->>T3: Read state
    T3->>S: Unlock
    T1->>T1: Read state
    T1->>S: Unlock
```

## State Management

### NetworkState Structure

```rust
pub struct NetworkState {
    // Link name <-> Index bidirectional mapping
    links: HashMap<String, u32>,           // name -> index
    link_names: HashMap<u32, String>,      // index -> name

    // Route tracking (key: interface_index + table_id)
    routes: HashMap<(u32, u32), Vec<Route>>,

    // Routing policy rules
    rules_from: HashMap<IpAddr, RoutingRule>,  // Source-based rules
    rules_to: HashMap<IpAddr, RoutingRule>,    // Dest-based rules

    // Interface state
    link_states: HashMap<u32, LinkState>,
}

pub struct LinkState {
    name: String,
    index: u32,
    flags: Vec<String>,
    operational_state: String,
    addresses: Vec<Address>,
    mtu: u32,
    mac: String,
}
```

### State Update Flow

```mermaid
stateDiagram-v2
    [*] --> EventReceived
    EventReceived --> AcquireReadLock: Check if change needed
    AcquireReadLock --> CompareCurrent: Read current state
    CompareCurrent --> ReleaseReadLock: No change
    ReleaseReadLock --> [*]: Skip processing

    CompareCurrent --> AcquireWriteLock: Change detected
    AcquireWriteLock --> ModifyState: Update data structures
    ModifyState --> ComputeActions: Determine actions
    ComputeActions --> ReleaseWriteLock: Save state
    ReleaseWriteLock --> ExecuteActions: Perform side effects
    ExecuteActions --> [*]: Done

    note right of AcquireReadLock
        Readers can proceed
        concurrently
    end note

    note right of AcquireWriteLock
        Exclusive access
        Blocks all others
    end note
```

### State Synchronization

```mermaid
sequenceDiagram
    participant NL as Netlink Event
    participant W as Watcher Task
    participant S as State (RwLock)
    participant R as Routing Manager
    participant E as Script Executor

    NL->>W: Address Added Event

    W->>S: Read Lock
    S-->>W: Current addresses
    W->>W: Compare with event

    alt No change detected
        W->>S: Release Lock
        W->>NL: Ignore event
    else Change detected
        W->>S: Release Read Lock
        W->>S: Write Lock
        S-->>W: Exclusive access
        W->>S: Update addresses
        W->>S: Release Lock

        par Parallel Actions
            W->>R: Configure routing
            W->>E: Execute scripts
        end
    end
```

## Event Processing Pipeline

### Pipeline Stages

```mermaid
graph LR
    subgraph "Stage 1: Reception"
        S1A[Netlink Socket]
        S1B[DBus Signal]
        S1C[File Notify]
    end

    subgraph "Stage 2: Parsing"
        S2A[Deserialize]
        S2B[Extract Fields]
        S2C[Type Conversion]
    end

    subgraph "Stage 3: Validation"
        S3A[Interface Filter]
        S3B[Event Type Check]
        S3C[Pattern Match]
    end

    subgraph "Stage 4: Processing"
        S4A[State Update]
        S4B[Diff Calculation]
        S4C[Action Planning]
    end

    subgraph "Stage 5: Execution"
        S5A[Route Config]
        S5B[Script Exec]
        S5C[Audit Log]
    end

    S1A & S1B & S1C --> S2A
    S2A --> S2B --> S2C
    S2C --> S3A --> S3B --> S3C
    S3C --> S4A --> S4B --> S4C
    S4C --> S5A & S5B & S5C

    style S3A fill:#ff9800,color:#fff
    style S4B fill:#ffeb3b
    style S5B fill:#4caf50,color:#fff
```

### Event Types and Handlers

```mermaid
graph TB
    EVENT[Network Event]

    EVENT -->|RTM_NEWADDR| ADDR_NEW[Address Added]
    EVENT -->|RTM_DELADDR| ADDR_DEL[Address Removed]
    EVENT -->|RTM_NEWLINK| LINK_NEW[Link Added/Changed]
    EVENT -->|RTM_DELLINK| LINK_DEL[Link Removed]
    EVENT -->|RTM_NEWROUTE| ROUTE_NEW[Route Added]
    EVENT -->|RTM_DELROUTE| ROUTE_DEL[Route Removed]
    EVENT -->|DBus Signal| DBUS[State Change]

    ADDR_NEW --> HANDLER_A[handle_address_new]
    ADDR_DEL --> HANDLER_B[handle_address_del]
    LINK_NEW --> HANDLER_C[handle_link_change]
    LINK_DEL --> HANDLER_D[handle_link_del]
    ROUTE_NEW --> HANDLER_E[handle_route_new]
    ROUTE_DEL --> HANDLER_F[handle_route_del]
    DBUS --> HANDLER_G[handle_state_change]

    HANDLER_A & HANDLER_B --> ACTION1[Update State]
    HANDLER_C & HANDLER_D --> ACTION2[Refresh Links]
    HANDLER_E & HANDLER_F --> ACTION3[Update Routes]
    HANDLER_G --> ACTION4[Process Backend Event]

    ACTION1 --> EXEC[Execute Actions]
    ACTION2 --> EXEC
    ACTION3 --> EXEC
    ACTION4 --> EXEC

    style EVENT fill:#2196f3,color:#fff
    style EXEC fill:#4caf50,color:#fff
```

## Backend Implementations

### systemd-networkd Backend

```mermaid
sequenceDiagram
    participant SD as systemd-networkd
    participant DB as DBus System Bus
    participant NE as netevd
    participant FS as /run/systemd/netif
    participant NL as Netlink

    Note over SD: Interface state change

    SD->>DB: PropertiesChanged signal
    DB->>NE: Receive signal
    NE->>NE: Extract link index

    par Parallel Data Fetch
        NE->>FS: Read /run/systemd/netif/links/N
        FS-->>NE: State file JSON

        NE->>NL: Query interface details
        NL-->>NE: Addresses, routes, flags
    end

    NE->>NE: Merge data
    NE->>NE: Build JSON payload
    NE->>NE: Update state
    NE->>NE: Execute routable.d scripts
```

### NetworkManager Backend

```mermaid
sequenceDiagram
    participant NM as NetworkManager
    participant DB as DBus System Bus
    participant NE as netevd
    participant SIG as Signal Matcher

    Note over NM: Device state change

    NM->>DB: StateChanged signal
    DB->>SIG: Deliver to subscribers
    SIG->>NE: Match & process

    NE->>NE: Parse device path
    NE->>DB: GetDeviceDetails(path)
    DB->>NM: Forward request
    NM-->>DB: Device properties
    DB-->>NE: Interface name, state

    alt State = Activated
        NE->>NE: Execute activated.d scripts
    else State = Disconnected
        NE->>NE: Execute disconnected.d scripts
    end
```

### dhclient Backend

```mermaid
sequenceDiagram
    participant DC as dhclient
    participant FS as File System
    participant NOT as inotify Watcher
    participant NE as netevd
    participant PARSE as Lease Parser

    DC->>FS: Write /var/lib/dhcp/dhclient.leases
    FS->>NOT: File modified event
    NOT->>NE: Notify change

    NE->>FS: Read lease file
    FS-->>NE: Lease data
    NE->>PARSE: Parse lease format
    PARSE-->>NE: Structured data

    NE->>NE: Extract: IP, gateway, DNS, domain
    NE->>NE: Update state
    NE->>NE: Execute routable.d scripts

    opt use_dns = true
        NE->>NE: Call systemd-resolved
    end

    opt use_hostname = true
        NE->>NE: Call systemd-hostnamed
    end
```

## Network Operations

### Routing Policy Rules Creation

```mermaid
flowchart TD
    START[Interface Routable Event]
    CHECK{Configured for<br/>policy rules?}
    START --> CHECK

    CHECK -->|No| END[Skip]
    CHECK -->|Yes| GETIP[Get IP Addresses]

    GETIP --> GETGW[Discover Gateway]
    GETGW --> CALCTBL[Calculate Table ID<br/>200 + ifindex]

    CALCTBL --> RULE1[Create Rule: from IP<br/>lookup table N]
    RULE1 --> RULE2[Create Rule: to IP<br/>lookup table N]
    RULE2 --> ROUTE[Add default route<br/>via gateway dev iface<br/>table N]

    ROUTE --> VERIFY{Rules created<br/>successfully?}
    VERIFY -->|Yes| LOG_OK[Log success]
    VERIFY -->|No| LOG_ERR[Log error]

    LOG_OK --> END
    LOG_ERR --> END

    style START fill:#2196f3,color:#fff
    style CALCTBL fill:#ffeb3b
    style ROUTE fill:#4caf50,color:#fff
```

### Address Change Handling

```mermaid
stateDiagram-v2
    [*] --> AddressEvent
    AddressEvent --> GetInterface: Extract ifindex
    GetInterface --> CheckMonitored: Lookup name

    CheckMonitored --> Ignore: Not monitored
    Ignore --> [*]

    CheckMonitored --> ProcessEvent: Monitored interface
    ProcessEvent --> UpdateState: Store address

    UpdateState --> CheckRoutingRules: Check config
    CheckRoutingRules --> CreateRules: Rules enabled
    CheckRoutingRules --> ExecuteScripts: Rules disabled

    CreateRules --> CalculateTableID
    CalculateTableID --> AddPolicyRules
    AddPolicyRules --> AddDefaultRoute
    AddDefaultRoute --> ExecuteScripts

    ExecuteScripts --> LogEvent
    LogEvent --> [*]

    note right of CreateRules
        Automatic routing
        table creation
    end note
```

## Security Architecture

### Privilege Drop Sequence

```mermaid
sequenceDiagram
    autonumber
    participant P as Process (UID=0)
    participant K as Kernel
    participant C as Capabilities

    Note over P: Started by root/systemd

    P->>K: prctl(PR_SET_KEEPCAPS, 1)
    Note over K: Capabilities will survive setuid

    P->>K: setgid(netevd)
    K-->>P: GID changed

    P->>K: setuid(netevd)
    K-->>P: UID changed (now unprivileged)

    Note over P: Now running as netevd user

    P->>K: prctl(PR_SET_KEEPCAPS, 0)
    Note over K: Disable further capability retention

    P->>C: capset(CLEAR_ALL)
    P->>C: capset(CAP_NET_ADMIN, PERMITTED)
    P->>C: capset(CAP_NET_ADMIN, EFFECTIVE)

    Note over P: Only CAP_NET_ADMIN retained

    P->>P: fork() for script execution
    Note over P: Child inherits NO capabilities
```

### Input Validation Layers

```mermaid
graph TD
    INPUT[External Input]

    INPUT --> L1[Layer 1: Type Validation]
    L1 -->|Interface Name| V1{Valid chars?<br/>alphanumeric, -, _, .}
    L1 -->|IP Address| V2{Valid IP?<br/>Parse with std::net}
    L1 -->|Hostname| V3{Valid hostname?<br/>RFC 1123}

    V1 -->|No| REJECT1[Reject]
    V2 -->|No| REJECT2[Reject]
    V3 -->|No| REJECT3[Reject]

    V1 -->|Yes| L2[Layer 2: Injection Check]
    V2 -->|Yes| L2
    V3 -->|Yes| L2

    L2 --> INJ{Contains shell<br/>metacharacters?}
    INJ -->|Yes: ; $ \` & |  | REJECT4[Reject]
    INJ -->|No| L3[Layer 3: Length Check]

    L3 --> LEN{Length within<br/>limits?}
    LEN -->|No| REJECT5[Reject]
    LEN -->|Yes| SANITIZE[Sanitize & Escape]

    SANITIZE --> SAFE[Safe to Use]

    REJECT1 & REJECT2 & REJECT3 & REJECT4 & REJECT5 --> LOG[Log Rejection]

    style INPUT fill:#2196f3,color:#fff
    style REJECT1 fill:#f44336,color:#fff
    style REJECT2 fill:#f44336,color:#fff
    style REJECT3 fill:#f44336,color:#fff
    style REJECT4 fill:#f44336,color:#fff
    style REJECT5 fill:#f44336,color:#fff
    style SAFE fill:#4caf50,color:#fff
```

## Performance Optimizations

### Memory Layout

```mermaid
graph TB
    subgraph "Heap (Arc + RwLock)"
        STATE[NetworkState<br/>~10-20 KB]
        CONFIG[Config<br/>~5 KB]
        METRICS[Metrics<br/>~15 KB]
    end

    subgraph "Stack Per Task"
        TASK1[Address Watcher<br/>~64 KB]
        TASK2[Route Watcher<br/>~64 KB]
        TASK3[Link Watcher<br/>~64 KB]
        TASK4[Backend Listener<br/>~64 KB]
    end

    subgraph "Total Memory"
        BINARY[Binary Code<br/>~2 MB]
        SHARED[Shared Libs<br/>~3 MB]
        HEAP[Heap Allocation<br/>~1-2 MB]
        STACK[Task Stacks<br/>~256 KB]
    end

    STATE -.-> HEAP
    CONFIG -.-> HEAP
    METRICS -.-> HEAP

    TASK1 -.-> STACK
    TASK2 -.-> STACK
    TASK3 -.-> STACK
    TASK4 -.-> STACK

    style STATE fill:#ffeb3b
    style HEAP fill:#4caf50,color:#fff
```

### Event Processing Performance

```mermaid
gantt
    title Event Processing Timeline
    dateFormat X
    axisFormat %L ms

    section Reception
    Netlink recv()           :0, 1

    section Parsing
    Deserialize              :1, 3

    section Validation
    Filter interface         :3, 4
    Check rules              :4, 5

    section State Update
    Acquire write lock       :5, 7
    Modify HashMap           :7, 9
    Release lock             :9, 10

    section Actions
    Configure routes         :10, 30
    Fork + exec script       :30, 100

    section Total
    Total latency            :milestone, 100, 100
```

### Optimization Techniques

1. **Lock Minimization**
   - Hold locks for minimum duration
   - Prefer read locks over write locks
   - Clone small data outside of locks

2. **Async I/O**
   - Non-blocking netlink sockets
   - Async DBus calls
   - Concurrent task execution

3. **Memory Efficiency**
   - Reuse buffers for netlink messages
   - HashMap for O(1) lookups
   - Arc for shared ownership (no cloning large data)

4. **Lazy Evaluation**
   - Only query netlink when needed
   - Cache interface index ↔ name mappings
   - Skip processing for unmonitored interfaces

## Module Dependencies

```mermaid
graph TB
    MAIN[main.rs]

    subgraph "Configuration"
        CONFIG[config/mod.rs]
    end

    subgraph "Network Layer"
        LINK[network/link.rs]
        ADDR[network/address.rs]
        ROUTE[network/route.rs]
        RULE[network/routing_rule.rs]
        WATCH[network/watcher.rs]
        NETMOD[network/mod.rs]
    end

    subgraph "Listeners"
        LNWD[listeners/networkd/]
        LNM[listeners/networkmanager/]
        LDH[listeners/dhclient/]
    end

    subgraph "System"
        USER[system/user.rs]
        CAP[system/capability.rs]
        EXEC[system/execute.rs]
        VALID[system/validation.rs]
    end

    subgraph "DBus"
        RESOLVED[bus/resolved.rs]
        HOSTNAMED[bus/hostnamed.rs]
    end

    MAIN --> CONFIG
    MAIN --> USER
    MAIN --> CAP
    MAIN --> WATCH
    MAIN --> LNWD
    MAIN --> LNM
    MAIN --> LDH

    WATCH --> NETMOD
    WATCH --> ADDR
    WATCH --> ROUTE
    WATCH --> LINK
    WATCH --> RULE

    LNWD --> NETMOD
    LNWD --> EXEC
    LNWD --> RESOLVED
    LNWD --> HOSTNAMED

    LNM --> NETMOD
    LNM --> EXEC

    LDH --> NETMOD
    LDH --> EXEC
    LDH --> RESOLVED
    LDH --> HOSTNAMED

    EXEC --> VALID
    ROUTE --> VALID
    RULE --> VALID

    style MAIN fill:#2196f3,color:#fff
    style NETMOD fill:#ffeb3b
    style VALID fill:#f44336,color:#fff
```

## Build and Compilation

### Build Pipeline

```mermaid
graph LR
    SRC[Source Code<br/>*.rs]
    RUSTC[rustc Compiler]
    OPT[Optimization<br/>LTO, stripped]
    LINK[Linker]
    BIN[Binary<br/>netevd]

    SRC --> RUSTC
    RUSTC -->|--release| OPT
    OPT --> LINK
    LINK --> BIN

    DEPS[Dependencies<br/>Cargo.toml]
    DEPS -.->|resolve| RUSTC

    style RUSTC fill:#2196f3,color:#fff
    style OPT fill:#ff9800,color:#fff
    style BIN fill:#4caf50,color:#fff
```

### Dependency Tree (Top-level)

```
netevd
├── tokio (async runtime)
│   ├── mio (system I/O)
│   └── parking_lot (synchronization)
├── rtnetlink (netlink operations)
│   ├── netlink-packet-route
│   └── futures
├── zbus (DBus async client)
│   └── zvariant
├── serde + serde_yaml (configuration)
├── nix (Unix system calls)
├── caps (Linux capabilities)
├── notify (file system events)
└── anyhow (error handling)
```

## See Also

- [README.md](../README.md) - Main documentation
- [API.md](API.md) - REST API reference
- [CONFIGURATION.md](../CONFIGURATION.md) - Configuration guide
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guide
