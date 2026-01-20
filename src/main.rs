// SPDX-License-Identifier: LGPL-3.0-or-later

//! netevd - Network Event Daemon
//!
//! A daemon that monitors systemd-networkd and dhclient events,
//! executes scripts on network state changes, and manages routing policy rules.

use anyhow::{Context, Result};
use rtnetlink::Handle;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod bus;
mod config;
mod listeners;
mod network;
mod system;

use config::Config;
use network::{link, watcher, NetworkState};
use system::user;

const DEFAULT_USER: &str = "netevd";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first
    init_logging();

    info!("Starting netevd - Network Event Daemon");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Parse configuration
    let config = Config::parse().context("Failed to parse configuration")?;
    info!(
        "Configuration loaded: generator={}, log_level={}",
        config.system.generator, config.system.log_level
    );

    // Update logging level based on config
    update_log_level(&config.system.log_level);

    // Drop privileges if running as root
    if user::is_root() {
        info!("Running as root, attempting to drop privileges to user '{}'", DEFAULT_USER);
        user::drop_privileges(DEFAULT_USER)
            .context("Failed to drop privileges")?;
    } else {
        info!("Not running as root, continuing with current user");
    }

    // Initialize network state
    let state = Arc::new(RwLock::new(NetworkState::new()));
    info!("Network state initialized");

    // Get netlink handle
    let handle = link::get_netlink_handle()
        .await
        .context("Failed to get netlink handle")?;
    info!("Netlink handle acquired");

    // Acquire initial links
    {
        let mut state_write = state.write().await;
        link::acquire_links(&mut state_write, &handle)
            .await
            .context("Failed to acquire initial links")?;
    }
    info!("Initial network links acquired");

    // Get routing policy interfaces from config
    let routing_policy_interfaces = config.network.get_routing_policy_interfaces();

    // Clone handles for async tasks
    let state_addr = state.clone();
    let state_route = state.clone();
    let state_link = state.clone();
    let state_listener = state.clone();
    let handle_addr = handle.clone();
    let handle_route = handle.clone();
    let handle_link = handle.clone();
    let handle_listener = handle.clone();
    let config_listener = config.clone();

    // Set up signal handlers
    let mut sigterm = signal(SignalKind::terminate())
        .context("Failed to set up SIGTERM handler")?;
    let mut sigint = signal(SignalKind::interrupt())
        .context("Failed to set up SIGINT handler")?;

    info!("netevd initialized successfully, waiting for events...");

    // Main event loop with async watchers
    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down gracefully");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT (Ctrl+C), shutting down gracefully");
        }
        result = watcher::watch_addresses(handle_addr, state_addr, routing_policy_interfaces) => {
            warn!("Address watcher exited: {:?}", result);
        }
        result = watcher::watch_routes(handle_route, state_route) => {
            warn!("Route watcher exited: {:?}", result);
        }
        result = watcher::watch_links(handle_link, state_link) => {
            warn!("Link watcher exited: {:?}", result);
        }
        result = spawn_listener(config_listener, handle_listener, state_listener) => {
            warn!("Generator listener exited: {:?}", result);
        }
    }

    info!("netevd shutdown complete");
    Ok(())
}

/// Initialize logging with default settings
fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .init();
}

/// Update logging level based on configuration
fn update_log_level(level: &str) {
    // Note: Dynamically updating log level requires using a reload handle
    // For now, we rely on RUST_LOG environment variable
    // TODO: Implement dynamic log level updates if needed
    match level {
        "trace" | "debug" | "info" | "warn" | "error" => {
            // Valid log level, would be applied if we had reload handle
        }
        _ => {
            warn!("Invalid log level '{}', using default", level);
        }
    }
}

/// Spawn the appropriate listener based on the configured generator
async fn spawn_listener(
    config: Config,
    handle: Handle,
    state: Arc<RwLock<NetworkState>>,
) -> Result<()> {
    match config.system.generator.as_str() {
        "systemd-networkd" => {
            info!("Starting systemd-networkd listener");
            listeners::networkd::listen_networkd(config, handle, state).await
        }
        "NetworkManager" => {
            info!("Starting NetworkManager listener");
            listeners::networkmanager::listen_networkmanager(config, handle, state).await
        }
        "dhclient" => {
            info!("Starting dhclient listener");
            listeners::dhclient::watch_lease_file(config, handle, state).await
        }
        _ => anyhow::bail!("Unknown generator: {}", config.system.generator),
    }
}
