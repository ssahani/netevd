# Testing Guide

This document describes the testing infrastructure for netevd.

## Test Coverage

**Total Tests: 50**
- Unit Tests: 42
- Integration Tests: 8

All tests passing ✅

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Only Unit Tests
```bash
cargo test --lib
```

### Run Only Integration Tests
```bash
cargo test --test integration_test
```

### Run Specific Test
```bash
cargo test test_name
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

## Unit Tests

Unit tests are located within the source files themselves using Rust's `#[cfg(test)]` annotation.

### Configuration Tests (`src/config/mod.rs`)
- ✅ `test_default_config` - Verify default configuration values
- ✅ `test_parse_links` - Parse space-separated link list
- ✅ `test_should_monitor_link` - Link monitoring logic
- ✅ `test_should_monitor_all_links_when_empty` - Monitor all when no filter

### Network Module Tests

#### Address Tests (`src/network/address.rs`)
- ✅ `test_is_link_local_ipv4` - IPv4 link-local detection (169.254.0.0/16)
- ✅ `test_is_link_local_ipv6` - IPv6 link-local detection (fe80::/10)

#### Link Tests (`src/network/link.rs`)
- ✅ `test_get_netlink_handle` - Netlink handle creation

#### Route Tests (`src/network/route.rs`)
- ✅ `test_calculate_table_id` - Custom routing table ID calculation (200 + ifindex)

#### Routing Rule Tests (`src/network/routing_rule.rs`)
- ✅ `test_table_calculation` - Verify table base is 200
- ✅ `test_route_table_base` - Constant value check

#### Network State Tests (`src/network/mod.rs`)
- ✅ `test_add_remove_link` - Link state tracking
- ✅ `test_routes` - Route management
- ✅ `test_routing_rules` - Routing policy rule tracking

### System Module Tests

#### Capability Tests (`src/system/capability.rs`)
- ✅ `test_capability_enum` - Capability constant availability
- ✅ `test_has_capability` - Capability checking function
- ✅ `test_capability_check_multiple` - Multiple capability checks
- ✅ `test_keep_capabilities_toggles` - PR_SET_KEEPCAPS toggling (root only)
- ✅ `test_apply_capabilities_non_root` - Graceful handling when not root

#### User Tests (`src/system/user.rs`)
- ✅ `test_is_root` - Root user detection
- ✅ `test_lookup_nonexistent_user` - Invalid user handling
- ✅ `test_lookup_root_user` - Root user lookup
- ✅ `test_drop_privileges_non_root` - Skip privilege drop when not root
- ✅ `test_lookup_user_empty_string` - Empty username validation
- ✅ `test_is_root_consistency` - UID check consistency

#### Validation Tests (`src/system/validation.rs`)
- ✅ `test_validate_interface_name` - Interface name validation
- ✅ `test_validate_ip_address` - IPv4/IPv6 address validation
- ✅ `test_validate_domain_name` - Domain name format validation
- ✅ `test_validate_hostname` - Hostname format validation
- ✅ `test_validate_state_name` - State name whitelist
- ✅ `test_sanitize_env_value` - Shell injection prevention
- ✅ `test_validate_ip_list` - IP address list validation

### Listener Tests

#### dhclient Parser Tests (`src/listeners/dhclient/parser.rs`)
- ✅ `test_extract_value` - Simple value extraction
- ✅ `test_extract_quoted_value` - Quoted string extraction
- ✅ `test_parse_valid_lease_file` - Full lease file parsing
- ✅ `test_parse_multiple_leases` - Multiple interface leases
- ✅ `test_parse_malformed_lease_file` - Missing interface handling
- ✅ `test_parse_empty_file` - Empty file handling
- ✅ `test_parse_nonexistent_file` - File not found error
- ✅ `test_extract_value_edge_cases` - Edge case handling
- ✅ `test_extract_quoted_value_edge_cases` - Quote parsing edge cases
- ✅ `test_parse_lease_with_comments` - Comment line handling
- ✅ `test_parse_lease_multiple_dns` - Comma-separated DNS servers

## Integration Tests

Integration tests are located in `tests/integration_test.rs` and test end-to-end functionality.

### Configuration Integration
- ✅ `test_config_loading` - Load and parse YAML configuration
- ✅ `test_invalid_config` - Error handling for malformed YAML
- ✅ `test_default_config` - Default value application
- ✅ `test_env_var_override` - Environment variable precedence

### Network State Integration
- ✅ `test_network_state_initialization` - NetworkState creation
- ✅ `test_script_dir_paths` - Script directory path generation
- ✅ `test_link_state_tracking` - Link add/query operations
- ✅ `test_concurrent_state_access` - Concurrent read/write with Arc<RwLock>

## Test Requirements

### Dependencies
The following dev dependencies are required:
```toml
[dev-dependencies]
tempfile = "3.8"  # Temporary file/directory creation for tests
```

### System Requirements
Some tests behave differently based on privileges:
- **Root Tests**: `test_keep_capabilities_toggles` only runs meaningful checks as root
- **Non-Root Tests**: `test_drop_privileges_non_root`, `test_apply_capabilities_non_root` verify graceful handling

## Writing New Tests

### Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Arrange
        let input = "test";

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Async Test Template
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Integration Test Template
```rust
#[test]
fn test_feature() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    // ... test code ...
}
```

## Continuous Integration

Tests are automatically run on:
- Every commit
- Pull requests
- Release builds

### CI Commands
```bash
# Run tests with all features
cargo test --all-features

# Run tests with coverage (requires tarpaulin)
cargo tarpaulin --out Html --output-dir coverage

# Check for test warnings
cargo test -- --nocapture 2>&1 | grep -i warning
```

## Test Coverage Goals

| Module | Coverage | Status |
|--------|----------|--------|
| `config` | 95%+ | ✅ |
| `network` | 85%+ | ✅ |
| `system` | 90%+ | ✅ |
| `listeners` | 80%+ | ✅ |
| `bus` | 70%+ | ⚠️ (No DBus mocking yet) |

## Known Test Limitations

1. **DBus Tests**: DBus listeners (`networkd`, `networkmanager`) require mocking - not yet implemented
2. **Netlink Tests**: Real netlink operations require elevated privileges
3. **File Watcher Tests**: `notify` crate file watching not tested in isolation
4. **Script Execution**: Script execution tests would require setting up test script directories

## Future Test Additions

- [ ] DBus mock tests for systemd-networkd listener
- [ ] DBus mock tests for NetworkManager listener
- [ ] Netlink event simulation tests
- [ ] File watcher event tests
- [ ] End-to-end daemon lifecycle tests
- [ ] Performance/benchmark tests
- [ ] Stress tests for concurrent netlink events
- [ ] Memory leak tests (valgrind/miri)

## Performance Tests

To benchmark critical paths:
```bash
cargo bench  # (requires benchmark configuration)
```

## Test Maintenance

- Update tests when adding new features
- Keep integration tests fast (<1s total)
- Use `#[ignore]` for slow tests that require special setup
- Document any test-specific environment setup

## Debugging Tests

### Run specific test with backtrace
```bash
RUST_BACKTRACE=1 cargo test test_name
```

### Run tests in single thread
```bash
cargo test -- --test-threads=1
```

### Show test output
```bash
cargo test -- --nocapture
```

---

**Last Updated**: 2026-01-20
**Test Count**: 50
**Pass Rate**: 100%
