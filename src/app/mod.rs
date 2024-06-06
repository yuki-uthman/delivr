use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::config::{Config, Environment, Zoho};
use crate::database::Database;
use crate::error::Result;
use crate::routes::build_router;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub zoho: Zoho,
}

pub async fn serve(config: &Config) -> Result<u16> {
    let pool = PgPool::connect(&config.database.connection_string()).await?;
    // run migrations
    Database::migrate(&pool).await?;

    let router = build_router(config).await;
    let listener = TcpListener::bind(config.addr()).await?;
    let port = listener.local_addr().unwrap().port();

    tracing::info!("Listening on {:?}", listener.local_addr()?);

    match config.environment {
        Environment::Test => {
            tokio::spawn(async {
                axum::serve(listener, router).await.unwrap();
            });
        }
        _ => {
            axum::serve(listener, router).await.unwrap();
        }
    }

    Ok(port)
}
