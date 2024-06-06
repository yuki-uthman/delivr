use axum::extract::{Path, State};
use axum::Json;
use axum::{http::StatusCode, response::IntoResponse};
use axum::{routing::get, Router};
use tower_http::trace::{self, TraceLayer};
use tracing::instrument;

use crate::app::AppState;
use crate::config::Config;
use crate::database::Tokens;
use crate::error::{Error, Result};

pub async fn build_router(config: &Config) -> Result<Router> {
    let state = AppState::build_state(config).await?;

    Ok(Router::new()
        .route("/health", get(health))
        .route("/token/:code", get(request_token))
        .route("/tokens", get(get_all_tokens))
        .route("/tokens/:scope", get(get_token))
        .route("/invoices", get(get_all_invoices))
        // Add a tracing layer to all requests
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new())
                .on_request(trace::DefaultOnRequest::new())
                .on_response(trace::DefaultOnResponse::new()),
        )
        .with_state(state))
}

#[instrument(skip(state))]
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

#[instrument(skip(state, code))]
pub async fn request_token(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("request token");

    if code.is_empty() {
        return Err(Error::custom("Missing code"));
    }

    let client = &state.client;
    let tokens = Tokens { pool: &state.pool };

    let token = client.request_token(&code).await?;
    if tokens.contains_scope(&token.scope).await? {
        tracing::warn!("token already exists, replacing the existing token");
        tokens.update(&token).await?;
    } else {
        tokens.insert(&token).await?;
    }

    tracing::info!("{:#?}", token);

    Ok(StatusCode::OK)
}

#[instrument(skip(state))]
pub async fn get_token(
    State(state): State<AppState>,
    Path(scope): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("get token for {}", scope);

    let tokens = Tokens { pool: &state.pool };
    let token = tokens.get_by_scope(&scope).await?;

    tracing::info!("{:#?}", token);

    Ok(Json(token))
}

#[instrument(skip(state))]
pub async fn get_all_tokens(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("get all tokens");

    let tokens = Tokens { pool: &state.pool };
    let tokens = tokens.get_all().await?;

    tracing::info!("{:#?}", tokens);

    Ok(Json(tokens))
}

#[instrument(skip(state))]
pub async fn get_all_invoices(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("get all invoices");

    let tokens = Tokens { pool: &state.pool };
    let mut token = tokens
        .get_by_scope("ZohoBooks.fullaccess.all")
        .await?
        .ok_or(Error::custom("No token found"))?;

    let client = &state.client;

    if token.is_expired() {
        tracing::info!("Token is expired, refreshing token...");

        token = client.refresh_token(&token).await.map_err(Error::from)?;

        tokens.update(&token).await?;

        tracing::info!("Token has been refreshed");
    }

    let value = client.get_all_invoices(&token).await?;

    Ok(Json(value))
}
