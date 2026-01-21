use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub interfaces_count: usize,
    pub routing_rules_count: usize,
    pub events_processed: u64,
    pub backend: String,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterfaceInfo {
    pub name: String,
    pub index: u32,
    pub state: String,
    pub addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub mtu: Option<u32>,
    pub flags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub destination: String,
    pub gateway: Option<IpAddr>,
    pub interface: String,
    pub metric: Option<u32>,
    pub table: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingRuleInfo {
    pub priority: u32,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub table: u32,
    pub interface: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub interface: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReloadRequest {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReloadResponse {
    pub success: bool,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub dbus: bool,
    pub netlink: bool,
    pub config: bool,
}
