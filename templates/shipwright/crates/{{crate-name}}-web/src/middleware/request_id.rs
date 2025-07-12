use axum::{
    extract::Request,
    http::{header::HeaderName, HeaderValue},
    response::Response,
};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use uuid::Uuid;

/// Layer that adds a unique request ID to each request
#[derive(Clone)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

/// Service that adds a unique request ID to each request
#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S> Service<Request> for RequestIdService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        // Generate a unique request ID
        let request_id = Uuid::new_v4().to_string();
        
        // Add the request ID to the request headers
        if let Ok(header_value) = HeaderValue::from_str(&request_id) {
            request.headers_mut().insert(
                HeaderName::from_static("x-request-id"),
                header_value,
            );
        }

        // Call the inner service
        self.inner.call(request)
    }
}