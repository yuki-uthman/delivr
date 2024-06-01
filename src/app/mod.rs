use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};

use crate::routes::health;

pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health", get(health))
        // Add a tracing layer to all requests
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new())
                .on_request(trace::DefaultOnRequest::new())
                .on_response(trace::DefaultOnResponse::new()),
        );

    let listener = TcpListener::bind("0.0.0.0:10000").await?;

    tracing::info!("listening on 0.0.0.0:10000");
    axum::serve(listener, app).await?;

    Ok(())
}
