use axum::{http::StatusCode, response::IntoResponse};
use tracing::instrument;

#[instrument]
pub async fn health() -> impl IntoResponse {
    tracing::info!("health check");
    StatusCode::OK.into_response()
}
