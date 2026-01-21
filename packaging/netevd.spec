Name:           netevd
Version:        0.1.0
Release:        1%{?dist}
Summary:        Network event daemon for systemd-networkd and NetworkManager

License:        LGPL-3.0-or-later
URL:            https://github.com/ssahani/netevd
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  make

Requires:       systemd
Requires(pre):  shadow-utils

%description
netevd is a high-performance network event daemon written in Rust that
configures network interfaces and executes scripts on network events from
systemd-networkd, NetworkManager DBus signals, or when dhclient gains a lease.

Features:
- Async/Await architecture built on tokio
- Multiple network manager support (systemd-networkd, NetworkManager, dhclient)
- Routing policy rules for multi-interface setups
- Script execution on network state changes
- Runs as unprivileged user with minimal capabilities (CAP_NET_ADMIN only)

%prep
%autosetup -p1

%build
cargo build --release --locked

%install
# Install binary
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install systemd service
install -Dm644 systemd/%{name}.service %{buildroot}%{_unitdir}/%{name}.service

# Install default configuration
install -Dm644 examples/%{name}.yaml %{buildroot}%{_sysconfdir}/%{name}/%{name}.yaml

# Create script directories
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/carrier.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/no-carrier.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/configured.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/degraded.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/routable.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/activated.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/disconnected.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/manager.d
install -dm755 %{buildroot}%{_sysconfdir}/%{name}/routes.d

# Install documentation
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 INSTALL.md %{buildroot}%{_docdir}/%{name}/INSTALL.md
install -Dm644 LICENSE %{buildroot}%{_docdir}/%{name}/LICENSE

%pre
# Create netevd user
getent group %{name} >/dev/null || groupadd -r %{name}
getent passwd %{name} >/dev/null || \
    useradd -r -g %{name} -d /nonexistent -s /usr/sbin/nologin \
    -c "Network Event Daemon" %{name}
exit 0

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun_with_restart %{name}.service

%files
%license LICENSE
%doc README.md INSTALL.md
%{_bindir}/%{name}
%{_unitdir}/%{name}.service
%dir %{_sysconfdir}/%{name}
%config(noreplace) %{_sysconfdir}/%{name}/%{name}.yaml
%dir %{_sysconfdir}/%{name}/carrier.d
%dir %{_sysconfdir}/%{name}/no-carrier.d
%dir %{_sysconfdir}/%{name}/configured.d
%dir %{_sysconfdir}/%{name}/degraded.d
%dir %{_sysconfdir}/%{name}/routable.d
%dir %{_sysconfdir}/%{name}/activated.d
%dir %{_sysconfdir}/%{name}/disconnected.d
%dir %{_sysconfdir}/%{name}/manager.d
%dir %{_sysconfdir}/%{name}/routes.d

%changelog
* Tue Jan 21 2026 Susant Sahani <ssahani@redhat.com> - 0.1.0-1
- Initial RPM release
- Support for systemd-networkd, NetworkManager, and dhclient
- Routing policy rules for multi-interface setups
- Security hardening with unprivileged user and minimal capabilities
