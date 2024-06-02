use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};
use axum::{routing::get, Router};
use sqlx::PgPool;
use tower_http::trace::{self, TraceLayer};
use tracing::instrument;

use crate::app::AppState;
use crate::error::{Error, Result};

pub fn build_router(pool: PgPool) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/health", get(health))
        // Add a tracing layer to all requests
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new())
                .on_request(trace::DefaultOnRequest::new())
                .on_response(trace::DefaultOnResponse::new()),
        )
        .with_state(state)
}

#[instrument]
pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("health check");

    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map_err(Error::Sqlx)?;

    if result == 1 {
        Ok(StatusCode::OK)
    } else {
        Err(Error::custom("Query did not return the expected result"))
    }
}
