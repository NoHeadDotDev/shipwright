use axum::{
    routing::get,
    Router,
};

use crate::{controllers::live_view, state::AppState};

/// Create the LiveView router with all LiveView endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(live_view::home::home_page))
        .route("/counter", get(live_view::counter::counter_page))
        .route("/dashboard", get(live_view::dashboard::dashboard_page))
}