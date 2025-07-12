use axum::{extract::State, response::Json};
use serde_json::{json, Value};

use crate::state::AppState;

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    // Basic health check - you can extend this to check database connectivity, etc.
    let health_status = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "database": "connected" // You could add actual DB health check here
    });

    Json(health_status)
}