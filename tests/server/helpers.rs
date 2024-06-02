use crate::error::Result;
use delivr::routes::build_router;
use delivr::app;
use delivr::config::{get_config, Config};

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

    let port = app::serve(&config).await?;

    config.application.port = port;

    Ok(App { config })
}
