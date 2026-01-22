# GitHub Configuration

This directory contains GitHub-specific configuration files for CI/CD, automation, and dependency management.

## Workflows

### CI Workflow (`workflows/ci.yml`)

Runs on every push to `main` and on all pull requests.

**Jobs:**
- **Test**: Runs all unit and integration tests
- **Clippy**: Runs Rust linter to catch common mistakes
- **Format**: Checks code formatting with rustfmt
- **Build**: Builds the project in release mode on stable and beta Rust
- **Security Audit**: Scans dependencies for known vulnerabilities

**Triggers:** Push to main, Pull requests

### Functional Tests Workflow (`workflows/functional-tests.yml`)

Runs comprehensive functional tests that create real network interfaces.

**Jobs:**
- **Functional Tests**: Runs 23 functional tests with sudo privileges
  - Creates dummy, veth, bridge, and macvlan interfaces
  - Tests IP address management (IPv4 and IPv6)
  - Tests MTU changes, routes, and statistics
  - Tests script execution and event handling
  - Verifies proper cleanup

**Triggers:** Push to main, Pull requests, Manual dispatch

### Coverage Workflow (`workflows/coverage.yml`)

Generates code coverage reports and uploads to Codecov.

**Jobs:**
- **Coverage**: Uses cargo-llvm-cov to generate coverage data
- Uploads LCOV report to Codecov

**Triggers:** Push to main, Pull requests

**Requirements:** `CODECOV_TOKEN` secret (optional, works without it)

### Release Workflow (`workflows/release.yml`)

Builds release binaries and publishes to GitHub releases and crates.io.

**Jobs:**
- **Create Release**: Creates GitHub release from tag
- **Build Release**: Builds binaries for:
  - x86_64-unknown-linux-gnu
  - aarch64-unknown-linux-gnu (ARM64)
- **Publish Crate**: Publishes to crates.io

**Triggers:** Push of tags matching `v*.*.*` (e.g., v0.2.0)

**Requirements:**
- `CARGO_REGISTRY_TOKEN` secret for crates.io publishing

## Dependabot Configuration (`dependabot.yml`)

Automatically creates pull requests to update dependencies.

**Updates:**
- Cargo dependencies: Weekly
- GitHub Actions: Weekly

**Labels:**
- `dependencies`
- `rust` or `github-actions`

## Secrets Required

Configure these in repository settings → Secrets and variables → Actions:

### Optional Secrets
- `CODECOV_TOKEN`: For uploading coverage reports to Codecov
  - Get from https://codecov.io/gh/ssahani/netevd
  - Works without token but may have rate limits

### Required for Releases
- `CARGO_REGISTRY_TOKEN`: For publishing to crates.io
  - Get from https://crates.io/me
  - Only needed when releasing new versions

## CI Badge Status

Add to README.md:

```markdown
[![CI](https://github.com/ssahani/netevd/actions/workflows/ci.yml/badge.svg)](https://github.com/ssahani/netevd/actions/workflows/ci.yml)
[![Functional Tests](https://github.com/ssahani/netevd/actions/workflows/functional-tests.yml/badge.svg)](https://github.com/ssahani/netevd/actions/workflows/functional-tests.yml)
[![codecov](https://codecov.io/gh/ssahani/netevd/branch/main/graph/badge.svg)](https://codecov.io/gh/ssahani/netevd)
```

## Local Testing

Test workflows locally before pushing:

### Run tests like CI does
```bash
# Install dependencies
sudo apt-get install -y libdbus-1-dev pkg-config

# Run tests
cargo test --verbose

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Build release
cargo build --release --verbose
```

### Run functional tests
```bash
# Build tests
cargo test --test functional_test --no-run

# Run with sudo
TEST_BINARY=$(find target/debug/deps -name "functional_test-*" -type f -executable | head -1)
sudo $TEST_BINARY --test-threads=1 --ignored
```

### Generate coverage locally
```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --all-features --workspace --html

# Open coverage report
open target/llvm-cov/html/index.html
```

## Workflow Triggers

| Workflow | Push to main | Pull Request | Tag | Manual |
|----------|--------------|--------------|-----|--------|
| CI | ✅ | ✅ | ❌ | ❌ |
| Functional Tests | ✅ | ✅ | ❌ | ✅ |
| Coverage | ✅ | ✅ | ❌ | ❌ |
| Release | ❌ | ❌ | ✅ | ❌ |

## Troubleshooting

### Functional tests fail
- Check that test interfaces are properly cleaned up
- Ensure Ubuntu runner has proper permissions
- Verify iproute2 is installed

### Release build fails
- Check that tag follows semver format (v0.2.0)
- Verify CARGO_REGISTRY_TOKEN is set
- Ensure Cargo.toml version matches tag

### Coverage upload fails
- Check CODECOV_TOKEN secret
- Verify repository is public or has Codecov integration
- Coverage uploads are not critical and can be skipped

## Maintenance

### Updating workflow versions
Dependabot automatically creates PRs for:
- GitHub Actions updates (e.g., actions/checkout@v4 → v5)
- Rust toolchain updates

### Adding new workflows
1. Create new `.yml` file in `workflows/`
2. Follow naming convention: `kebab-case.yml`
3. Add appropriate triggers and jobs
4. Test locally if possible
5. Update this README

### Removing workflows
1. Delete the workflow file
2. Remove badges from README.md
3. Update this documentation

### Documentation Workflow (`workflows/docs.yml`)

Builds and deploys documentation to GitHub Pages.

**Jobs:**
- **Docs**: Builds rustdoc with strict warnings
  - Deploys to GitHub Pages on main branch
- **Spelling**: Checks spelling with typos
- **Links**: Verifies all links in markdown and source files

**Triggers:** Push to main, Pull requests

**Output:** https://ssahani.github.io/netevd/

### Security Workflow (`workflows/security.yml`)

Comprehensive security scanning.

**Jobs:**
- **CodeQL Analysis**: Static code analysis for security vulnerabilities
- **Trivy Scan**: Container and filesystem vulnerability scanning
- **Supply Chain**: License and dependency checks with cargo-deny

**Triggers:** Push to main, Pull requests, Daily at 2 AM UTC

**Requires:** Security events write permission

### Benchmark Workflow (`workflows/benchmark.yml`)

Performance benchmarking and regression detection.

**Jobs:**
- **Benchmarks**: Runs cargo benchmarks
  - Stores results for comparison
  - Alerts on 150%+ performance regression
- **Memory Profiling**: Tracks binary size and memory usage

**Triggers:** Push to main, Pull requests, Manual dispatch

### Cross Platform Workflow (`workflows/cross-platform.yml`)

Tests on multiple distributions and Rust versions.

**Jobs:**
- **Distro Tests**: Tests on Ubuntu, Debian, Fedora, Arch Linux
- **MSRV**: Minimum Supported Rust Version (1.70)
- **Nightly**: Tests against Rust nightly (allowed to fail)

**Triggers:** Push to main, Pull requests, Weekly on Sundays

### Docker Workflow (`workflows/docker.yml`)

Builds and publishes container images.

**Jobs:**
- **Build and Push**: Creates Debian and Alpine images
  - Pushes to GitHub Container Registry (ghcr.io)
  - Multi-platform support
- **Container Scan**: Security scanning of built images

**Triggers:** Push to main, Version tags, Pull requests

**Images:**
- `ghcr.io/ssahani/netevd:latest-debian`
- `ghcr.io/ssahani/netevd:latest-alpine`
- `ghcr.io/ssahani/netevd:v0.2.0-debian`

### PR Automation Workflow (`workflows/pr-automation.yml`)

Automates pull request management.

**Jobs:**
- **Auto Label**: Labels PRs based on changed files
- **Size Labeler**: Adds size labels (xs, s, m, l, xl)
- **Conventional Commits**: Validates commit messages
- **PR Title**: Validates semantic PR titles
- **Welcome**: Greets first-time contributors

**Triggers:** PR opened, synchronized, labeled

### Stale Workflow (`workflows/stale.yml`)

Manages stale issues and pull requests.

**Jobs:**
- **Stale**: Marks and closes inactive items
  - Issues: Stale after 90 days, closed after 14
  - PRs: Stale after 60 days, closed after 14
  - Exempts pinned and security items

**Triggers:** Daily at 1 AM UTC, Manual dispatch

## Configuration Files

### Commit Linting (`.commitlintrc.json`)

Enforces conventional commit format:
- Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert, config
- Max header length: 100 characters
- No uppercase in subject

### Auto Labeling (`labeler.yml`)

Automatically labels PRs based on changed files:
- `area: documentation` - Markdown files
- `area: config` - Configuration files
- `area: network` - Network code
- `area: tests` - Test files
- `area: ci` - CI/CD files
- `area: security` - Security-related code
- `area: api` - API and DBus code
- `area: cli` - CLI code
- `dependencies` - Cargo files

### Dependency Policy (`deny.toml`)

Cargo-deny configuration:
- **Advisories**: Deny vulnerabilities, warn on unmaintained
- **Licenses**: Allow MIT, Apache-2.0, BSD, ISC, LGPL-3.0-or-later
- **Bans**: Warn on duplicate versions
- **Sources**: Only allow crates.io registry

### Docker Configuration

**Dockerfiles:**
- `Dockerfile.debian` - ~150MB, better compatibility
- `Dockerfile.alpine` - ~50MB, minimal footprint

**Features:**
- Multi-stage builds
- Non-root user
- Capability support
- Health checks
- Optimized layers

`.dockerignore` excludes unnecessary files from build context.

