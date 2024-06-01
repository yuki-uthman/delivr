use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};

use crate::config::Config;
use crate::error::Result;
use crate::routes::health;

pub async fn serve(config: Config) -> Result<()> {
    let app = Router::new()
        .route("/health", get(health))
        // Add a tracing layer to all requests
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new())
                .on_request(trace::DefaultOnRequest::new())
                .on_response(trace::DefaultOnResponse::new()),
        );

    let addr = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("listening on {:?}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
