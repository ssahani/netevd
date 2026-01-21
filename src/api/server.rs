use crate::api::handlers::AppState;
use crate::api::routes::create_api_routes;
use crate::network::NetworkState;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
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
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = create_api_routes(self.state)
            .layer(cors)
            .layer(TraceLayer::new_for_http());

        tracing::info!("Starting API server on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
