use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::config::{Config, Environment};
use crate::error::Result;
use crate::routes::build_router;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
}

pub async fn serve(config: &Config) -> Result<u16> {
    let pool = PgPool::connect(&config.database.connection_string()).await?;
    let router = build_router(pool);
    let listener = TcpListener::bind(config.addr()).await?;
    let port = listener.local_addr().unwrap().port();

    tracing::info!("Listening on {:?}", listener.local_addr()?);

    // check the APP_ENVIRONMENT
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    // wrap the axum::serve in tokio::spawn if testing
    if environment == Environment::Test {
        tokio::spawn(async {
            axum::serve(listener, router).await.unwrap();
        });
    } else {
        axum::serve(listener, router).await.unwrap();
    }

    Ok(port)
}
