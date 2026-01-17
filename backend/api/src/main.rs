use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing with JSON formatting
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Investment Research Platform API...");

    // build our application with a route
    let app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive()) // specific config recommended for prod
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "healthy" }))
}
