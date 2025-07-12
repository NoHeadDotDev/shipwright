//! Controllers module for handling HTTP requests
//! 
//! This module contains all the controller logic for handling HTTP requests
//! and responses. Controllers are responsible for:
//! - Processing incoming requests
//! - Validating input data
//! - Calling business logic
//! - Returning appropriate responses
//!
//! ## Usage
//!
//! Controllers are typically used with the web framework router:
//!
//! ```rust
//! use axum::{Router, routing::get};
//! use crate::controllers::health;
//!
//! let app = Router::new()
//!     .route("/health", get(health::health_check));
//! ```

pub mod health;
pub mod users;

use axum::{
    response::{Json, IntoResponse},
    http::StatusCode,
};
use serde_json::json;
use yeah_ship_shared::AppError;

/// Standard JSON error response
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Convert AppError to HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(ErrorResponse {
            error: status.canonical_reason().unwrap_or("Unknown error").to_string(),
            message: error_message.to_string(),
        });

        (status, body).into_response()
    }
}

/// Helper function to create success responses
pub fn success_response<T: serde::Serialize>(data: T) -> impl IntoResponse {
    Json(json!({
        "success": true,
        "data": data
    }))
}

/// Helper function to create paginated responses
pub fn paginated_response<T: serde::Serialize>(
    data: Vec<T>,
    page: u32,
    per_page: u32,
    total: u64,
) -> impl IntoResponse {
    let total_pages = (total + per_page as u64 - 1) / per_page as u64;
    
    Json(json!({
        "success": true,
        "data": data,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
            "total_pages": total_pages,
            "has_next": page < total_pages as u32,
            "has_prev": page > 1
        }
    }))
}