use axum::{extract::State, response::Json};
use serde_json::{json, Value};

use crate::state::AppState;

/// Get application version information
pub async fn get_version(State(state): State<AppState>) -> Json<Value> {
    let version_info = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "rust_version": env!("CARGO_PKG_RUST_VERSION"),
        "build_time": env!("CARGO_PKG_VERSION"), // You could add actual build time here
    });

    Json(version_info)
}