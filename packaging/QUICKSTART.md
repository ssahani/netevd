# Quick Start: Package Building

Quick reference for building packages for different distributions.

## RPM (Fedora/RHEL/CentOS)

```bash
# One-time setup
sudo dnf install rpm-build rust cargo
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Build
git archive --format=tar.gz --prefix=netevd-0.1.0/ -o ~/rpmbuild/SOURCES/netevd-0.1.0.tar.gz HEAD
cp packaging/netevd.spec ~/rpmbuild/SPECS/
rpmbuild -ba ~/rpmbuild/SPECS/netevd.spec

# Install
sudo dnf install ~/rpmbuild/RPMS/x86_64/netevd-0.1.0-1.*.rpm
sudo systemctl enable --now netevd
```

## DEB (Debian/Ubuntu)

```bash
# One-time setup
sudo apt-get install debhelper dh-cargo cargo rustc build-essential

# Build
cp -r packaging/debian .
dpkg-buildpackage -us -uc -b

# Install
sudo dpkg -i ../netevd_0.1.0-1_amd64.deb
sudo systemctl enable --now netevd
```

## AUR (Arch Linux)

```bash
# Publish (maintainers only)
git clone ssh://aur@aur.archlinux.org/netevd.git aur-netevd
cd aur-netevd
cp /path/to/packaging/PKGBUILD .
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to 0.1.0"
git push

# Install (users)
yay -S netevd
# or
git clone https://aur.archlinux.org/netevd.git && cd netevd && makepkg -si
```

## Cargo/crates.io

```bash
# Publish (maintainers only)
cargo login
cargo publish

# Install (users)
cargo install netevd
# Then follow manual setup in packaging/README.md
```

## Testing Before Release

```bash
# All platforms
cargo test
cargo clippy
cargo build --release

# RPM specific
rpmlint ~/rpmbuild/RPMS/*/netevd-*.rpm

# DEB specific
lintian ../netevd_*.deb

# AUR specific
namcap PKGBUILD
```

## Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update version in packaging/netevd.spec
- [ ] Update packaging/debian/changelog
- [ ] Update packaging/PKGBUILD (pkgver)
- [ ] Update packaging/.SRCINFO
- [ ] Run all tests: `cargo test`
- [ ] Build all packages and test installation
- [ ] Create git tag: `git tag -a v0.1.0 -m "Release 0.1.0"`
- [ ] Push tag: `git push origin v0.1.0`
- [ ] Publish to crates.io: `cargo publish`
- [ ] Update AUR repository
- [ ] Create GitHub release with binaries
