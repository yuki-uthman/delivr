use axum::extract::{Path, State};
use axum::Json;
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
        .route("/token/:code", get(token))
        .route("/tokens/:scope", get(get_token))
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

use crate::zoho::Token;

#[instrument]
pub async fn token(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("token");

    if code.is_empty() {
        return Err(Error::custom("Missing code"));
    }

    let id = std::env::var("APP_ZOHO__CLIENT_ID").expect("APP_ZOHO_CLIENT_ID must be set");
    let secret =
        std::env::var("APP_ZOHO__CLIENT_SECRET").expect("APP_ZOHO_CLIENT_SECRET must be set");

    let response = reqwest::Client::new()
        .post("https://accounts.zoho.com/oauth/v2/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("client_id", &id),
            ("client_secret", &secret),
        ])
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(error) = response.get("error") {
        return Err(Error::custom(format!("Zoho error: {error}")));
    }

    let token = Token::from(response);
    token.insert(&state.pool).await?;

    tracing::info!("{:#?}", token);

    Ok(StatusCode::OK)
}

#[instrument]
pub async fn get_token(
    State(state): State<AppState>,
    Path(scope): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("get token for {}", scope);

    let db_pool = &state.pool;

    let res = sqlx::query_as::<_, Token>(
        r#"
        SELECT
            scope,
            access_token,
            api_domain,
            expires_in,
            refresh_token,
            token_type,
            time_stamp
        FROM tokens
        WHERE scope = $1
        "#,
    )
    .bind(scope)
    .fetch_one(db_pool)
    .await;

    if let Err(err) = res {
        tracing::error!("{:#?}", err);
        return Err(Error::Sqlx(err));
    }

    let token = res.unwrap();

    tracing::info!("{:#?}", token);

    Ok(Json(token))
}
