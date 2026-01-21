# Packaging Guide for netevd

This directory contains packaging files for multiple Linux distributions and package managers.

## Table of Contents

- [RPM Package (Fedora/RHEL/openSUSE)](#rpm-package-fedorarhel)
- [DEB Package (Debian/Ubuntu)](#deb-package-debianubuntu)
- [AUR Package (Arch Linux)](#aur-package-arch-linux)
- [Cargo/crates.io](#cargocrates-io)
- [Building from Source](#building-from-source)

---

## RPM Package (Fedora/RHEL)

### Prerequisites

```bash
# Install build tools
sudo dnf install rpm-build rust cargo systemd-rpm-macros
```

### Build RPM

```bash
# From the project root directory
cd /path/to/netevd

# Create RPM build environment
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create source tarball
git archive --format=tar.gz --prefix=netevd-0.1.0/ -o ~/rpmbuild/SOURCES/netevd-0.1.0.tar.gz HEAD

# Copy spec file
cp packaging/netevd.spec ~/rpmbuild/SPECS/

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/netevd.spec
```

### Install RPM

```bash
# Install the generated RPM
sudo dnf install ~/rpmbuild/RPMS/x86_64/netevd-0.1.0-1.fc*.x86_64.rpm

# Start the service
sudo systemctl enable --now netevd
```

### Verify Installation

```bash
# Check service status
sudo systemctl status netevd

# View logs
sudo journalctl -u netevd -f
```

---

## DEB Package (Debian/Ubuntu)

### Prerequisites

```bash
# Install build tools
sudo apt-get update
sudo apt-get install debhelper dh-cargo cargo rustc build-essential
```

### Build DEB

```bash
# From the project root directory
cd /path/to/netevd

# Copy debian directory to project root (if not already there)
cp -r packaging/debian .

# Build the package
dpkg-buildpackage -us -uc -b

# Or use debuild
debuild -us -uc -b
```

### Install DEB

```bash
# Install the generated .deb file
sudo dpkg -i ../netevd_0.1.0-1_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f

# Start the service
sudo systemctl enable --now netevd
```

### Verify Installation

```bash
# Check service status
sudo systemctl status netevd

# View package info
dpkg -l | grep netevd

# List installed files
dpkg -L netevd
```

---

## AUR Package (Arch Linux)

### Publishing to AUR

1. **Clone the AUR repository:**

```bash
# First time setup
git clone ssh://aur@aur.archlinux.org/netevd.git aur-netevd
cd aur-netevd
```

2. **Copy PKGBUILD and .SRCINFO:**

```bash
# Copy from packaging directory
cp /path/to/netevd/packaging/PKGBUILD .
cp /path/to/netevd/packaging/.SRCINFO .
```

3. **Update checksums:**

```bash
# Generate tarball checksum
makepkg -g >> PKGBUILD

# Or manually update sha256sums in PKGBUILD
```

4. **Update .SRCINFO:**

```bash
makepkg --printsrcinfo > .SRCINFO
```

5. **Commit and push to AUR:**

```bash
git add PKGBUILD .SRCINFO
git commit -m "Initial import of netevd 0.1.0"
git push origin master
```

### Installing from AUR

#### Using yay:

```bash
yay -S netevd
```

#### Using paru:

```bash
paru -S netevd
```

#### Manual installation:

```bash
# Clone AUR repository
git clone https://aur.archlinux.org/netevd.git
cd netevd

# Build and install
makepkg -si
```

### Verify Installation

```bash
# Check service status
sudo systemctl status netevd

# View package info
pacman -Qi netevd
```

---

## Cargo/crates.io

### Publishing to crates.io

**Note:** The Cargo.toml is already configured with all necessary metadata for crates.io.

1. **Login to crates.io:**

```bash
cargo login
```

2. **Verify the package builds:**

```bash
cargo build --release
cargo test
cargo clippy
```

3. **Dry-run publish:**

```bash
cargo publish --dry-run
```

4. **Publish to crates.io:**

```bash
cargo publish
```

### Installing from crates.io

```bash
# Install directly from crates.io
cargo install netevd

# The binary will be installed to ~/.cargo/bin/netevd
```

### Post-Installation Setup (Cargo install)

After installing via cargo, you need to set up the system manually:

```bash
# Create user
sudo useradd --system --no-create-home --shell /usr/sbin/nologin netevd

# Create config directory
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}

# Copy example config (you'll need to clone the repo for this)
sudo curl -o /etc/netevd/netevd.yaml https://raw.githubusercontent.com/ssahani/netevd/main/examples/netevd.yaml

# Copy and install systemd service
sudo curl -o /lib/systemd/system/netevd.service https://raw.githubusercontent.com/ssahani/netevd/main/systemd/netevd.service

# Copy binary to system location
sudo cp ~/.cargo/bin/netevd /usr/bin/netevd

# Reload systemd and start
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
```

---

## Building from Source

### Prerequisites

- Rust 1.70 or later
- Cargo
- Linux with systemd

### Build Steps

```bash
# Clone repository
git clone https://github.com/ssahani/netevd.git
cd netevd

# Build release binary
cargo build --release

# Run tests
cargo test

# Install manually
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service
sudo install -Dm644 examples/netevd.yaml /etc/netevd/netevd.yaml

# Create script directories
sudo mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}

# Create user
sudo useradd --system --no-create-home --shell /usr/sbin/nologin netevd

# Start service
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
```

---

## Distribution Matrix

| Distribution | Package Type | Installation Method | Recommended |
|--------------|--------------|---------------------|-------------|
| **Fedora** | RPM | `dnf install netevd*.rpm` | ✅ Yes |
| **RHEL/CentOS** | RPM | `yum install netevd*.rpm` | ✅ Yes |
| **openSUSE** | RPM | `zypper install netevd*.rpm` | ✅ Yes |
| **Debian** | DEB | `dpkg -i netevd*.deb` | ✅ Yes |
| **Ubuntu** | DEB | `dpkg -i netevd*.deb` | ✅ Yes |
| **Arch Linux** | AUR | `yay -S netevd` | ✅ Yes |
| **Manjaro** | AUR | `pamac install netevd` | ✅ Yes |
| **Any Linux** | Cargo | `cargo install netevd` | ⚠️ Manual setup needed |
| **Any Linux** | Source | Build from source | ⚠️ Manual setup needed |

---

## Package Maintainer Notes

### Version Updates

When releasing a new version:

1. **Update version in Cargo.toml:**
   ```toml
   version = "0.2.0"
   ```

2. **Update RPM spec file:**
   - Update `Version:` field
   - Add entry to `%changelog` section

3. **Update Debian changelog:**
   ```bash
   dch -v 0.2.0-1 "New upstream release"
   ```

4. **Update PKGBUILD:**
   - Update `pkgver` variable
   - Update checksum
   - Regenerate `.SRCINFO`

5. **Tag the release:**
   ```bash
   git tag -a v0.2.0 -m "Release version 0.2.0"
   git push origin v0.2.0
   ```

### Testing Packages

Before releasing:

```bash
# RPM
rpmlint ~/rpmbuild/RPMS/x86_64/netevd-*.rpm

# DEB
lintian ../netevd_*.deb

# AUR
namcap PKGBUILD
```

---

## Security Considerations

All packages should:
- Create the `netevd` system user
- Set appropriate file permissions (755 for directories, 644 for config)
- Install systemd service with security hardening
- Grant only CAP_NET_ADMIN capability

---

## Support

For packaging issues:
- **GitHub Issues:** https://github.com/ssahani/netevd/issues
- **Email:** Susant Sahani <ssahani@redhat.com>

---

## License

All packaging scripts are released under LGPL-3.0-or-later, same as netevd.
