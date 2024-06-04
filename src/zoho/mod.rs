use secrecy::{ExposeSecret, Secret};
use serde::{ser::SerializeStruct, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

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

impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Token", 6)?;
        state.serialize_field("access_token", &self.access_token.expose_secret())?;
        state.serialize_field("api_domain", &self.api_domain)?;
        state.serialize_field("expires_in", &self.expires_in)?;
        state.serialize_field(
            "refresh_token",
            &self.refresh_token.as_ref().map(|rt| rt.expose_secret()),
        )?;
        state.serialize_field("scope", &self.scope)?;
        state.serialize_field("token_type", &self.token_type)?;
        state.serialize_field("time_stamp", &self.time_stamp)?;
        state.end()
    }
}

impl<'r> FromRow<'r, PgRow> for Token {
    fn from_row(row: &'r PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Token {
            access_token: Secret::new(row.try_get("access_token")?),
            api_domain: row.try_get("api_domain")?,
            expires_in: row.try_get("expires_in")?,
            refresh_token: row
                .try_get("refresh_token")
                .map(|opt: Option<String>| opt.map(Secret::new))?,
            scope: row.try_get("scope")?,
            token_type: row.try_get("token_type")?,
            time_stamp: row.try_get("time_stamp")?,
        })
    }
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
