use tokio::net::TcpListener;

use crate::config::Config;
use crate::error::Result;
use crate::routes::build_router;

pub async fn serve(config: Config) -> Result<()> {
    let router = build_router();
    let listener = TcpListener::bind(config.addr()).await?;

    tracing::info!("listening on {:?}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}
