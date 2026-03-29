<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->

# Contributing

## Quick Setup

```bash
git clone https://github.com/YOUR_USERNAME/netevd.git
cd netevd
git remote add upstream https://github.com/ssahani/netevd.git

rustup component add rustfmt clippy
cargo build && cargo test && cargo clippy -- -D warnings
```

## Development Workflow

1. Create a branch: `git checkout -b feature/my-change`
2. Make changes
3. Verify:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
4. Commit: `git commit -m "component: brief description"`
5. Push and open a PR

## Commit Messages

```
component: Brief description of change

Longer explanation of what and why (if needed).

Fixes #123
```

Components: `network`, `config`, `listeners`, `api`, `cli`, `system`, `docs`, `tests`

## Code Standards

- Run `cargo fmt` before committing
- All `cargo clippy` warnings must be clean
- Follow Rust naming conventions (`snake_case` functions, `CamelCase` types)
- Add rustdoc comments for public APIs
- Validate all external input using functions from `system/validation.rs`
- Never use `sh -c` with user data -- use `Command::new()` directly

## Testing

```bash
# Unit and integration tests
cargo test

# Functional tests (require root -- create real network interfaces)
cargo test --test functional_test --no-run
sudo target/debug/deps/functional_test-* --test-threads=1 --ignored

# Manual testing
cargo build --release
sudo env RUST_LOG=debug target/release/netevd --config examples/netevd.yaml
```

## Project Structure

```
src/
├── main.rs              # Entry point, event loop, privilege drop
├── config/              # YAML config parsing
├── network/             # State management, netlink watchers, routing
├── listeners/           # Backend event listeners (networkd, NM, dhclient)
├── bus/                 # DBus integration (resolved, hostnamed)
├── system/              # Security (capabilities, user, validation, exec)
├── api/                 # REST API (Axum)
├── cli/                 # CLI commands
├── metrics/             # Prometheus metrics
├── audit/               # Audit logging
├── filters/             # Event filtering
└── cloud/               # Cloud provider integration
```

## What to Work On

Check the [issue tracker](https://github.com/ssahani/netevd/issues) for:
- `good first issue` -- newcomer-friendly tasks
- `help wanted` -- community contributions welcome

See [ROADMAP.md](ROADMAP.md) for planned features and priorities.

## Pull Request Checklist

- [ ] Code compiles (`cargo build`)
- [ ] Tests pass (`cargo test`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`)
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (for user-facing changes)

## Code of Conduct

Be respectful, welcoming, and constructive. Focus on the code, not the person. We're all here to build something useful.

## License

By contributing, you agree that your code will be licensed under [LGPL-3.0-or-later](https://www.gnu.org/licenses/lgpl-3.0.html). All source files must include `// SPDX-License-Identifier: LGPL-3.0-or-later`.

## Questions?

Open an issue or email ssahani@redhat.com.
