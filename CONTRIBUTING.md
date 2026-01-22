<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Contributing to netevd

Thank you for your interest in contributing to netevd! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of experience level, gender, gender identity and expression, sexual orientation, disability, personal appearance, body size, race, ethnicity, age, religion, or nationality.

### Expected Behavior

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- Rust 1.70 or later installed
- Git for version control
- A GitHub account
- Basic knowledge of:
  - Rust programming
  - Linux networking (netlink, routing)
  - systemd or NetworkManager (optional)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/netevd.git
cd netevd
```

3. Add upstream remote:

```bash
git remote add upstream https://github.com/ssahani/netevd.git
```

4. Create a branch for your work:

```bash
git checkout -b feature/my-new-feature
```

## Development Setup

### Install Development Tools

```bash
# Install Rust toolchain components
rustup component add rustfmt clippy

# Install additional tools (optional)
cargo install cargo-watch
cargo install cargo-audit
```

### Build the Project

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Watch mode (auto-rebuild on changes)
cargo watch -x build
```

### Run Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Code Quality Checks

```bash
# Format code
cargo fmt

# Check formatting without making changes
cargo fmt --check

# Run clippy (linter)
cargo clippy

# Run clippy with strict warnings
cargo clippy -- -D warnings

# Security audit
cargo audit
```

## How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug fixes**: Fix issues reported in the issue tracker
- **New features**: Add new functionality
- **Documentation**: Improve or add documentation
- **Tests**: Add or improve test coverage
- **Examples**: Add example scripts or configurations
- **Performance**: Optimize existing code
- **Refactoring**: Improve code structure or readability

### Finding Something to Work On

1. Check the [issue tracker](https://github.com/ssahani/netevd/issues)
2. Look for issues labeled:
   - `good first issue`: Great for newcomers
   - `help wanted`: Need community assistance
   - `bug`: Confirmed bugs
   - `enhancement`: Feature requests

3. Comment on the issue to let others know you're working on it

## Coding Standards

### Rust Style Guide

Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/):

- Use `rustfmt` for automatic formatting
- Follow Rust naming conventions:
  - `snake_case` for functions and variables
  - `CamelCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Code Organization

```
src/
├── main.rs           # Entry point
├── lib.rs           # Library exports
├── config/          # Configuration parsing
├── network/         # Network operations
├── listeners/       # Event listeners
├── bus/            # DBus integration
└── system/         # System operations
```

### Documentation

- Add rustdoc comments for public APIs:

```rust
/// Retrieves the netlink handle for network operations.
///
/// # Errors
///
/// Returns an error if the netlink connection cannot be established.
///
/// # Examples
///
/// ```
/// let handle = get_netlink_handle().await?;
/// ```
pub async fn get_netlink_handle() -> Result<Handle> {
    // Implementation
}
```

- Keep comments concise and meaningful
- Update documentation when changing functionality

### Error Handling

Use `anyhow` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

pub fn process_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read configuration file")?;

    serde_yaml::from_str(&content)
        .context("Failed to parse YAML configuration")
}
```

### Security

- Validate all external inputs
- Never use user input directly in shell commands
- Use existing validation functions from `system/validation.rs`
- Follow the principle of least privilege
- Document security implications

Example:
```rust
use crate::system::validation::validate_interface_name;

pub fn configure_interface(name: &str) -> Result<()> {
    // Always validate input
    validate_interface_name(name)?;

    // Safe to use now
    // ...
}
```

## Testing

### Writing Tests

Add tests for new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let yaml = r#"
system:
  log_level: "info"
  backend: "systemd-networkd"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.system.log_level, "info");
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = my_async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Place integration tests in `tests/`:

```rust
// tests/integration_test.rs
use netevd::config::Config;

#[test]
fn test_full_workflow() {
    // Test complete workflows
}
```

### Manual Testing

Test manually before submitting:

```bash
# Build and install locally
cargo build --release
sudo install -Dm755 target/release/netevd /usr/local/bin/netevd-test

# Run with test config
sudo /usr/local/bin/netevd-test --config test-config.yaml

# Check logs
sudo journalctl -f
```

## Pull Request Process

### Before Submitting

1. **Ensure code compiles**:
   ```bash
   cargo build
   cargo build --release
   ```

2. **Run tests**:
   ```bash
   cargo test
   ```

3. **Check formatting**:
   ```bash
   cargo fmt --check
   ```

4. **Run clippy**:
   ```bash
   cargo clippy -- -D warnings
   ```

5. **Update documentation**:
   - Update README.md if needed
   - Add rustdoc comments for new APIs
   - Update CHANGELOG.md

6. **Commit message format**:
   ```
   component: Brief description of change

   Longer explanation of what changed and why.

   Fixes #123
   ```

   Examples:
   - `network: Add support for IPv6 routing rules`
   - `config: Fix YAML parsing for empty values`
   - `docs: Update installation instructions`

### Submitting Pull Request

1. **Push to your fork**:
   ```bash
   git push origin feature/my-new-feature
   ```

2. **Create pull request** on GitHub

3. **Fill out PR template**:
   - Description of changes
   - Related issue numbers
   - Testing performed
   - Screenshots (if UI changes)

4. **Respond to reviews**:
   - Address reviewer comments
   - Push additional commits if needed
   - Be open to feedback

### After Submission

- Your PR will be reviewed by maintainers
- CI checks must pass
- At least one maintainer approval required
- Maintainers may request changes
- Once approved, maintainers will merge

## Reporting Bugs

### Before Reporting

1. **Search existing issues**: Your bug might already be reported
2. **Use latest version**: Ensure you're using the most recent release
3. **Reproduce the bug**: Verify it's reproducible

### Bug Report Template

Create an issue with:

**Title**: Clear, concise description

**Environment**:
- OS: (e.g., Ubuntu 22.04)
- Rust version: (e.g., 1.70.0)
- netevd version: (e.g., 0.2.0)
- Network backend: (e.g., systemd-networkd)

**Description**: What happened vs. what you expected

**Steps to Reproduce**:
1. Step one
2. Step two
3. Step three

**Configuration**:
```yaml
# Your netevd.yaml
```

**Logs**:
```
# Relevant log output
sudo journalctl -u netevd -n 100
```

**Additional Context**: Any other relevant information

## Feature Requests

### Before Requesting

1. **Search existing issues**: Feature might already be requested
2. **Check documentation**: Feature might already exist
3. **Consider scope**: Should it be in core or a plugin?

### Feature Request Template

**Title**: Clear feature description

**Problem**: What problem does this solve?

**Proposed Solution**: How should it work?

**Alternatives**: What alternatives have you considered?

**Use Cases**: When would you use this?

**Examples**: Code or configuration examples

## Development Tips

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable trace logging for specific module
RUST_LOG=netevd::network=trace cargo run

# Run with backtrace
RUST_BACKTRACE=1 cargo run
```

### Profiling

```bash
# Build with debug symbols
cargo build --release --profile release-with-debug

# Use perf (Linux)
perf record -g target/release/netevd
perf report
```

### Documentation

```bash
# Build and view docs
cargo doc --open

# Build docs with private items
cargo doc --document-private-items --open
```

## License

By contributing to netevd, you agree that your contributions will be licensed under the LGPL-3.0-or-later license.

All source files must include the SPDX license identifier:

```rust
// SPDX-License-Identifier: LGPL-3.0-or-later
```

For documentation files:
```markdown
<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->
```

## Questions?

- Open an issue for questions
- Discussion forum: (if available)
- Email: ssahani@redhat.com

## Thank You!

Your contributions make netevd better for everyone. We appreciate your time and effort!
