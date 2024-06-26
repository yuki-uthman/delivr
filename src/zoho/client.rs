use secrecy::{ExposeSecret, Secret};
use tracing::instrument;

use crate::config::Config;
use crate::zoho::{Error, Query, Result, Token};

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub client_secret: Secret<String>,
    pub client: reqwest::Client,
}

impl Client {
    pub fn new(config: &Config) -> Self {
        let client_id = config.zoho.client_id.clone();
        let client_secret = config.zoho.client_secret.clone();

        Self {
            client_id,
            client_secret,
            client: reqwest::Client::new(),
        }
    }

    pub async fn request_token(&self, code: &str) -> Result<Token> {
        tracing::info!("--> Zoho");
        let response = self
            .client
            .post("https://accounts.zoho.com/oauth/v2/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", &self.client_id),
                ("client_secret", self.client_secret.expose_secret()),
            ])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(error) = response.get("error") {
            return Err(Error::response(format!("{error}")));
        }

        tracing::info!("<-- Zoho 200");
        Ok(response.into())
    }

    pub async fn refresh_token(&self, token: &Token) -> Result<Token> {
        tracing::info!("--> Zoho");
        let refresh_token = token.refresh_token.as_ref().unwrap();
        let response = self
            .client
            .post("https://accounts.zoho.com/oauth/v2/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.expose_secret()),
                ("client_id", &self.client_id),
                ("client_secret", self.client_secret.expose_secret()),
            ])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(error) = response.get("error") {
            return Err(Error::response(format!("{error}")));
        }

        let mut token = Token::from(response);
        token.refresh_token = Some(refresh_token.clone());

        tracing::info!("<-- Zoho 200");
        Ok(token)
    }

    #[instrument(skip(self, token, query))]
    pub async fn get_invoices_with_query<'a>(
        &self,
        token: &Token,
        query: &'a Query<'a>,
    ) -> Result<serde_json::Value> {
        tracing::info!("--> Zoho");

        let res = self
            .client
            .get("https://www.zohoapis.com/books/v3/invoices")
            .header(
                "Authorization",
                format!("Zoho-oauthtoken {}", token.access_token.expose_secret()),
            )
            .query(&query)
            .send()
            .await;

        let res = match res {
            Ok(res) => {
                if res.status().is_success() {
                    tracing::info!("<-- Zoho: {}", res.status());
                    res.json::<serde_json::Value>().await
                } else {
                    let res = res.json::<serde_json::Value>().await;
                    let msg = res.unwrap()["message"].as_str().unwrap().to_string();
                    return Err(Error::response(msg));
                }
            }
            Err(err) => {
                tracing::error!("{err:#?}");
                return Err(Error::from(err));
            }
        };

        let value = match res {
            Ok(res) => res,
            Err(err) => {
                tracing::error!("{err:#?}");
                return Err(Error::from(err));
            }
        };

        Ok(value)
    }

    #[instrument(skip(self, token, id, query))]
    pub async fn get_invoice<'a>(
        &self,
        token: &Token,
        id: &'a str,
        query: &'a Query<'a>,
    ) -> Result<serde_json::Value> {
        tracing::info!("--> Zoho");

        let res = self
            .client
            .get(format!("https://www.zohoapis.com/books/v3/invoices/{id}"))
            .header(
                "Authorization",
                format!("Zoho-oauthtoken {}", token.access_token.expose_secret()),
            )
            .query(&query)
            .send()
            .await;

        let res = match res {
            Ok(res) => {
                if res.status().is_success() {
                    res.json::<serde_json::Value>().await
                } else {
                    let res = res.json::<serde_json::Value>().await;
                    let msg = res.unwrap()["message"].as_str().unwrap().to_string();
                    return Err(Error::response(msg));
                }
            }
            Err(err) => {
                tracing::error!("{err:#?}");
                return Err(Error::from(err));
            }
        };

        let value = match res {
            Ok(res) => {
                if res.get("invoice").is_some() {
                    res.get("invoice").unwrap().clone()
                } else {
                    return Err(Error::custom("Invoice not found in the response"));
                }
            }
            Err(err) => {
                tracing::error!("{err:#?}");
                return Err(Error::from(err));
            }
        };

        tracing::info!("<-- Zoho 200");
        Ok(value)
    }
}
