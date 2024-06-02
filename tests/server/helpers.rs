use crate::error::Result;
use delivr::app;
use delivr::config::{get_config, Config};
use sqlx::{Connection, PgConnection, Row};

pub struct App {
    config: Config,
}

impl App {
    pub fn url(&self) -> String {
        format!(
            "http://{}:{}",
            self.config.application.host, self.config.application.port
        )
    }
}

async fn check_database(config: &Config) -> Result<()> {
    let mut connection = PgConnection::connect(&config.database.connection_string_without_db())
        .await
        .map_err(|_| "Failed to connect to Postgres")?;

    let select_query = "SELECT 1";

    let row = sqlx::query(select_query)
        .fetch_one(&mut connection)
        .await
        .map_err(|_| format!("Failed to execute query: {}", select_query))?;

    let value: i32 = row
        .try_get(0)
        .map_err(|_| "Failed to retrieve query result")?;

    if value != 1 {
        return Err(format!(
            "Query did not return the expected result: {} -> {}",
            select_query, value
        )
        .into());
    }

    Ok(())
}

async fn create_database(config: &Config) -> Result<()> {
    let mut connection = PgConnection::connect(&config.database.connection_string_without_db())
        .await
        .map_err(|_| "Failed to connect to Postgres")?;

    let query_string = format!(r#"CREATE DATABASE "{}";"#, config.database.database_name);

    sqlx::query(&query_string)
        .execute(&mut connection)
        .await
        .map_err(|_| format!("Failed to execute query: {}", query_string))?;

    Ok(())
}

async fn setup_database(config: &Config) -> Result<()> {
    check_database(config).await?;
    create_database(config).await?;

    Ok(())
}

pub async fn setup_app() -> Result<App> {
    // set APP_ENVIRONMENT
    std::env::set_var("APP_ENVIRONMENT", "test");
    let mut config = get_config()?;

    config.database.database_name = uuid::Uuid::new_v4().to_string();
    setup_database(&config).await?;

    config.application.port = 0;
    let port = app::serve(&config).await?;

    config.application.port = port;

    Ok(App { config })
}
