use axum::{
    routing::{get, post},
    Router,
};

use crate::{controllers::api, state::AppState};

/// Create the API router with all API endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(api::users::list_users).post(api::users::create_user))
        .route("/users/{id}", get(api::users::get_user).put(api::users::update_user).delete(api::users::delete_user))
        .route("/version", get(api::version::get_version))
}