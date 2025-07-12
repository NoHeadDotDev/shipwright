use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

/// List all users
pub async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    // TODO: Implement actual database query
    // For now, return a mock response
    let users = vec![
        UserResponse {
            id: Uuid::new_v4().to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    Ok(Json(users))
}

/// Get a specific user by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserResponse>, StatusCode> {
    // TODO: Implement actual database query
    // For now, return a mock response
    let user = UserResponse {
        id: id.clone(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(user))
}

/// Create a new user
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // TODO: Implement actual database insertion
    // For now, return a mock response
    let user = UserResponse {
        id: Uuid::new_v4().to_string(),
        name: payload.name,
        email: payload.email,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(user))
}

/// Update an existing user
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // TODO: Implement actual database update
    // For now, return a mock response
    let user = UserResponse {
        id: id.clone(),
        name: payload.name.unwrap_or_else(|| "John Doe".to_string()),
        email: payload.email.unwrap_or_else(|| "john@example.com".to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(user))
}

/// Delete a user
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement actual database deletion
    // For now, return a success response
    Ok(Json(json!({ "message": "User deleted successfully" })))
}