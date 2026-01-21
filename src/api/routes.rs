use crate::api::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_api_routes(state: AppState) -> Router {
    Router::new()
        // API v1 routes
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/interfaces", get(list_interfaces))
        .route("/api/v1/interfaces/:name", get(get_interface))
        .route("/api/v1/routes", get(list_routes))
        .route("/api/v1/rules", get(list_rules))
        .route("/api/v1/events", get(list_events))
        .route("/api/v1/reload", post(reload_config))
        // Health and metrics
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .with_state(state)
}
