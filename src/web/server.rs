//! Web server setup and routing

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::info;

use crate::config::Config;
use super::{handlers, state::AppState};

/// Run the web server
pub async fn run_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = format!("{}:{}", config.web.bind_address, config.web.port);

    info!("Starting Marvinous web dashboard on {}", bind_addr);

    // Create shared application state
    let state = Arc::new(AppState::new(config.clone()));

    // Build router with all endpoints
    let app = Router::new()
        // API routes
        .route("/api/reports", get(handlers::list_reports))
        .route("/api/reports/:filename", get(handlers::get_report))
        .route("/api/collect", post(handlers::trigger_collect))
        .route("/api/status", get(handlers::get_status))
        .route("/health", get(handlers::health_check))
        // Serve static files from system location
        .nest_service("/", ServeDir::new("/usr/share/marvinous/static"))
        // Add shared state
        .with_state(state)
        // Add middleware
        .layer(TraceLayer::new_for_http());

    // Bind and serve
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Web server listening on http://{}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
