use jiff::civil::DateTime;

#[derive(Debug, Clone)]
pub struct Login {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: String,
    pub expires: DateTime,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
}
