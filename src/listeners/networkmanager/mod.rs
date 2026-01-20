// SPDX-License-Identifier: LGPL-3.0-or-later

//! NetworkManager DBus listener

pub mod dbus;

// Re-export the main listener function
pub use dbus::listen_networkmanager;
