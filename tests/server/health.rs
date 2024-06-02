use crate::error::Result;
use crate::helpers::setup_app;

#[tokio::test]
async fn health() -> Result<()> {
    let app = setup_app().await?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", app.url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    Ok(())
}
