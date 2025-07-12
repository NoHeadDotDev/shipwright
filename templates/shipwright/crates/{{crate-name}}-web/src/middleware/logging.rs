use axum::{extract::Request, response::Response};
use std::{
    task::{Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};
use tracing::{info, warn};

/// Layer that logs request information
#[derive(Clone)]
pub struct LoggingLayer;

impl LoggingLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

/// Service that logs request information
#[derive(Clone)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S> Service<Request> for LoggingService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start_time = Instant::now();
        let method = request.method().clone();
        let uri = request.uri().clone();
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let future = self.inner.call(request);

        Box::pin(async move {
            let response = future.await?;
            let duration = start_time.elapsed();
            let status = response.status();

            if status.is_client_error() || status.is_server_error() {
                warn!(
                    method = %method,
                    uri = %uri,
                    status = %status,
                    duration_ms = duration.as_millis(),
                    request_id = %request_id,
                    "Request completed with error"
                );
            } else {
                info!(
                    method = %method,
                    uri = %uri,
                    status = %status,
                    duration_ms = duration.as_millis(),
                    request_id = %request_id,
                    "Request completed"
                );
            }

            Ok(response)
        })
    }
}