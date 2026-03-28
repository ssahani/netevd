use crate::api::handlers::AppState;
use crate::api::routes::create_api_routes;
use crate::network::NetworkState;
use anyhow::Result;
use axum::http::{HeaderValue, Method};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub struct ApiServer {
    addr: SocketAddr,
    state: AppState,
}

impl ApiServer {
    pub fn new(port: u16, state: Arc<RwLock<NetworkState>>) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        Self { addr, state }
    }

    pub async fn run(self) -> Result<()> {
        let cors = CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
                origin.to_str().map_or(false, |s| {
                    s.starts_with("http://127.0.0.1")
                        || s.starts_with("http://localhost")
                        || s.starts_with("http://[::1]")
                })
            }))
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([axum::http::header::CONTENT_TYPE]);

        let app = create_api_routes(self.state)
            .layer(cors)
            .layer(TraceLayer::new_for_http());

        tracing::info!("Starting API server on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
