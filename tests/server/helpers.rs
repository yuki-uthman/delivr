use crate::error::Result;
use delivr::routes::build_router;
use delivr::config::{get_config, Config};

async fn start_app(config: &Config) -> Result<u16> {
    let router = build_router();
    let listener = tokio::net::TcpListener::bind(config.addr()).await.unwrap();

    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async {
        axum::serve(listener, router).await.unwrap();
    });

    Ok(port)
}

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

pub async fn setup_app() -> Result<App> {
    let mut config = get_config()?;

    let port = start_app(&config).await?;

    config.application.port = port;

    Ok(App { config })
}
