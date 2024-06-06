mod tokens;
pub use tokens::Tokens;

use crate::error::{Error, Result};
use sqlx::PgPool;

pub struct Database;

impl Database {
    pub async fn migrate(pool: &PgPool) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .map_err(|_| Error::custom("Failed to run migrations"))?;
        Ok(())
    }
}
