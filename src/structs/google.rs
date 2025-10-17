use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleLogin {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_in: Option<i64>,
    pub id_token: String,
    pub scope: String,
    pub token_type: String,
}
