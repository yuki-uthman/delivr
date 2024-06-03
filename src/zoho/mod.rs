use secrecy::Secret;

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
