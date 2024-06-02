use axum::{http::StatusCode, response::IntoResponse};
use axum::{routing::get, Router};
use tower_http::trace::{self, TraceLayer};
use tracing::instrument;

use crate::error::Result;

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        // Add a tracing layer to all requests
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new())
                .on_request(trace::DefaultOnRequest::new())
                .on_response(trace::DefaultOnResponse::new()),
        )
}

#[instrument]
pub async fn health() -> Result<impl IntoResponse> {
    tracing::info!("health check");

    Ok(StatusCode::OK)
}
