use crate::api::models::*;
use crate::network::NetworkState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type AppState = Arc<RwLock<NetworkState>>;

/// GET /api/v1/status
pub async fn get_status(State(state): State<AppState>) -> Json<ApiResponse<DaemonStatus>> {
    let state = state.read().await;

    let status = DaemonStatus {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: Track actual uptime
        interfaces_count: state.links_by_name.len(),
        routing_rules_count: state.routing_rules_from.len() + state.routing_rules_to.len(),
        events_processed: 0, // TODO: Track events
        backend: "systemd-networkd".to_string(), // TODO: Get from config
        dry_run: false, // TODO: Get from config
    };

    Json(ApiResponse::success(status))
}

/// GET /api/v1/interfaces
pub async fn list_interfaces(State(state): State<AppState>) -> Json<ApiResponse<Vec<InterfaceInfo>>> {
    let state = state.read().await;

    let interfaces: Vec<InterfaceInfo> = state
        .links_by_name
        .iter()
        .map(|(name, index)| InterfaceInfo {
            name: name.clone(),
            index: *index,
            state: "up".to_string(), // TODO: Get actual state
            addresses: vec![], // TODO: Get addresses
            mac_address: None, // TODO: Get MAC
            mtu: None, // TODO: Get MTU
            flags: vec![], // TODO: Get flags
        })
        .collect();

    Json(ApiResponse::success(interfaces))
}

/// GET /api/v1/interfaces/:name
pub async fn get_interface(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<InterfaceInfo>>, StatusCode> {
    let state = state.read().await;

    let interface = state
        .links_by_name
        .get(&name)
        .map(|index| InterfaceInfo {
            name: name.clone(),
            index: *index,
            state: "up".to_string(),
            addresses: vec![],
            mac_address: None,
            mtu: None,
            flags: vec![],
        });

    match interface {
        Some(iface) => Ok(Json(ApiResponse::success(iface))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// GET /api/v1/routes
pub async fn list_routes(State(_state): State<AppState>) -> Json<ApiResponse<Vec<RouteInfo>>> {
    // TODO: Implement route listing
    let routes = vec![];
    Json(ApiResponse::success(routes))
}

/// GET /api/v1/rules
pub async fn list_rules(State(state): State<AppState>) -> Json<ApiResponse<Vec<RoutingRuleInfo>>> {
    let state = state.read().await;

    let mut rules: Vec<RoutingRuleInfo> = Vec::new();

    // Add "from" rules
    for (addr, rule) in state.routing_rules_from.iter() {
        rules.push(RoutingRuleInfo {
            priority: 32765, // TODO: Get actual priority
            source: Some(addr.to_string()),
            destination: None,
            table: rule.table,
            interface: None,
        });
    }

    // Add "to" rules
    for (addr, rule) in state.routing_rules_to.iter() {
        rules.push(RoutingRuleInfo {
            priority: 32766, // TODO: Get actual priority
            source: None,
            destination: Some(addr.to_string()),
            table: rule.table,
            interface: None,
        });
    }

    Json(ApiResponse::success(rules))
}

/// GET /api/v1/events
pub async fn list_events(State(_state): State<AppState>) -> Json<ApiResponse<Vec<NetworkEvent>>> {
    // TODO: Implement event history
    let events = vec![];
    Json(ApiResponse::success(events))
}

/// POST /api/v1/reload
pub async fn reload_config(
    State(_state): State<AppState>,
    Json(_payload): Json<ReloadRequest>,
) -> Json<ReloadResponse> {
    // TODO: Implement config reload
    Json(ReloadResponse {
        success: true,
        message: "Configuration reloaded successfully".to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// GET /health
pub async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "healthy".to_string(),
        checks: HealthChecks {
            dbus: true,    // TODO: Check actual DBus connection
            netlink: true, // TODO: Check actual netlink connection
            config: true,  // TODO: Check config validity
        },
    })
}

/// GET /metrics
pub async fn metrics() -> String {
    // TODO: Return Prometheus metrics
    "# HELP netevd_info netevd daemon information\n\
     # TYPE netevd_info gauge\n\
     netevd_info{version=\"0.1.0\"} 1\n"
        .to_string()
}
