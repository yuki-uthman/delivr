use axum::extract::{Path, State};
use axum::Json;
use axum::{http::StatusCode, response::IntoResponse};
use axum::{routing::get, Router};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use tower_http::trace::{self, TraceLayer};
use tracing::instrument;

use crate::app::AppState;
use crate::database::Tokens;
use crate::error::{Error, Result};

pub fn build_router(pool: PgPool) -> Router {
    let state = AppState { pool };

    Router::new()
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
        .with_state(state)
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

use crate::zoho::Token;

#[instrument(skip(state, code))]
pub async fn request_token(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("request token");

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

    let tokens = Tokens { pool: &state.pool };

    let token = Token::from(response);
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
    let client = reqwest::Client::new();

    let tokens = Tokens { pool: &state.pool };
    let token = tokens
        .get_by_scope("ZohoBooks.fullaccess.all")
        .await?
        .ok_or(Error::custom("No token found"))?;

    let res = client
        .get("https://www.zohoapis.com/books/v3/invoices")
        .header(
            "Authorization",
            format!("Zoho-oauthtoken {}", token.access_token.expose_secret()),
        )
        .query(&[("organization_id", &String::from("820117212"))])
        .send()
        .await;

    let res = match res {
        Ok(res) => res.json::<serde_json::Value>().await,
        Err(err) => {
            tracing::error!("{err:#?}");
            return Err(Error::from(err));
        }
    };

    let value = match res {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("{err:#?}");
            return Err(Error::from(err));
        }
    };

    Ok(Json(value))
}
