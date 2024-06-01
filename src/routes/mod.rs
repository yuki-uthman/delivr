use axum::{http::StatusCode, response::IntoResponse};
use tracing::instrument;

use crate::error::Result;

#[instrument]
pub async fn health() -> Result<impl IntoResponse> {
    tracing::info!("health check");

    Ok(StatusCode::OK)
}
