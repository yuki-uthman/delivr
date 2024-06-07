use delivr::config::get_config;
use delivr::{app, config::Environment};
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // check app environment
    if std::env::var("APP_ENVIRONMENT").is_err() {
        std::env::set_var("APP_ENVIRONMENT", "local");
    }

    let config = get_config()?;

    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match config.environment {
        Environment::Production => {
            let subscriber = Registry::default().with(env_filter).with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .without_time(),
            );

            set_global_default(subscriber).unwrap();
        }
        _ => {
            let subscriber = Registry::default()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().with_ansi(true));

            set_global_default(subscriber).unwrap();
        }
    }

    tracing::info!("{:#?}", config);
    app::serve(&config).await?;

    Ok(())
}
