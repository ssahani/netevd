// SPDX-License-Identifier: LGPL-3.0-or-later

//! systemd-networkd DBus listener

pub mod api;
pub mod dbus;
pub mod json;

// Re-export the main listener function
pub use dbus::listen_networkd;
