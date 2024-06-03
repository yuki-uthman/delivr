use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Token {
    pub access_token: Secret<String>,
    pub api_domain: String,
    pub expires_in: i64,
    pub refresh_token: Option<Secret<String>>,
    pub scope: String,
    pub token_type: String,
    pub time_stamp: chrono::DateTime<chrono::Utc>,
}

impl From<serde_json::Value> for Token {
    fn from(val: serde_json::Value) -> Self {
        let now = chrono::Utc::now();

        let refresh_token = if val["refresh_token"].is_null() {
            None
        } else {
            Some(val["refresh_token"].as_str().unwrap().to_string())
        };

        Self {
            access_token: val["access_token"].as_str().unwrap().to_string().into(),
            api_domain: val["api_domain"].as_str().unwrap().to_string(),
            expires_in: val["expires_in"].as_i64().unwrap(),
            refresh_token: refresh_token.map(|s| s.into()),
            scope: val["scope"].as_str().unwrap().to_string(),
            token_type: val["token_type"].as_str().unwrap().to_string(),
            time_stamp: now,
        }
    }
}

impl Token {
    pub async fn insert(&self, pool: &PgPool) -> Result<()> {
        let query = r#"
            INSERT INTO tokens (access_token, api_domain, expires_in, refresh_token, scope, token_type, time_stamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        let access_token = self.access_token.expose_secret();
        let refresh_token = self.refresh_token.as_ref().map(|rt| rt.expose_secret());

        let mut conn = pool.acquire().await?;
        let res = sqlx::query(query)
            .bind(access_token)
            .bind(&self.api_domain)
            .bind(self.expires_in)
            .bind(refresh_token)
            .bind(&self.scope)
            .bind(&self.token_type)
            .bind(self.time_stamp)
            .execute(&mut *conn)
            .await
            .map_err(Error::from)?;

        if res.rows_affected() != 1 {
            return Err(Error::custom("Failed to insert token"));
        }

        Ok(())
    }
}
