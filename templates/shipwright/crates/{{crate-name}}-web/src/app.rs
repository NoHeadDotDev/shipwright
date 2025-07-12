use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};

use crate::{middleware, routes, state::AppState};

/// Create the main application router with all routes and middleware
pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors)
        .layer(middleware::request_id::RequestIdLayer::new())
        .layer(middleware::logging::LoggingLayer::new());

    Router::new()
        // API routes
        .nest("/api", routes::api::router())
        // LiveView routes
        .nest("/live", routes::live_view::router())
        // Static file serving
        .nest_service("/assets", ServeDir::new("assets"))
        // Shipwright LiveView JavaScript bundle
        .route("/bundle.js", shipwright_liveview::precompiled_js())
        // Health check
        .route("/health", axum::routing::get(routes::health::health_check))
        // Apply middleware stack
        .layer(middleware_stack)
        // Add application state
        .with_state(state)
}