use crate::error::{Error, Result};
use crate::zoho::Token;
use secrecy::ExposeSecret;
use sqlx::{PgPool};

pub struct Tokens<'a> {
    pub pool: &'a PgPool,
}

impl<'a> Tokens<'a> {
    pub async fn insert(&self, token: &Token) -> Result<()> {
        let query = r#"
            INSERT INTO tokens (access_token, api_domain, expires_in, refresh_token, scope, token_type, time_stamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        let access_token = token.access_token.expose_secret();
        let refresh_token = token.refresh_token.as_ref().map(|rt| rt.expose_secret());

        let mut conn = self.pool.acquire().await?;
        let res = sqlx::query(query)
            .bind(access_token)
            .bind(&token.api_domain)
            .bind(token.expires_in)
            .bind(refresh_token)
            .bind(&token.scope)
            .bind(&token.token_type)
            .bind(token.time_stamp)
            .execute(&mut *conn)
            .await
            .map_err(Error::from)?;

        if res.rows_affected() != 1 {
            return Err(Error::custom("Failed to insert token"));
        }

        Ok(())
    }

    pub async fn get_by_scope(&self, scope: &str) -> Result<Token> {
        let query = r#"
            SELECT access_token, api_domain, expires_in, refresh_token, scope, token_type, time_stamp
            FROM tokens
            WHERE scope = $1
        "#;

        let mut conn = self.pool.acquire().await?;
        let res = sqlx::query_as::<_, Token>(query)
            .bind(scope)
            .fetch_optional(&mut *conn)
            .await
            .map_err(Error::from)?;

        if res.is_none() {
            tracing::error!("No token found: {scope}");
            return Err(Error::custom(format!("No token found: {scope}")));
        }

        Ok(res.unwrap())
    }

    pub async fn get_all(&self) -> Result<Vec<Token>> {
        let query = r#"
            SELECT access_token, api_domain, expires_in, refresh_token, scope, token_type, time_stamp
            FROM tokens
        "#;

        let mut conn = self.pool.acquire().await?;
        let res = sqlx::query_as::<_, Token>(query)
            .fetch_all(&mut *conn)
            .await;

        if let Err(err) = res {
            tracing::error!("{:#?}", err);
            return Err(Error::Sqlx(err));
        }

        Ok(res.unwrap())
    }

}
