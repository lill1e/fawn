mod structs;

use jiff::{ToSpan, Zoned};
pub use structs::{fawn::*, google::*};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FawnError {
    #[error("request")]
    Request(#[from] ureq::Error),
    #[error("json")]
    JSON(#[from] serde_json::Error),
    #[error("date")]
    Date(#[from] jiff::Error),
    #[error("google login")]
    Google,
}

impl GoogleLogin {
    fn wrap(&self, client_id: String, client_secret: String) -> Result<Login, FawnError> {
        let cl = self.clone();
        dbg!(&cl);
        match cl.refresh_token {
            Some(r_token) => Ok(Login {
                client_id: client_id,
                client_secret: client_secret,
                access_token: cl.access_token,
                expires: Zoned::now().datetime() + self.expires_in.seconds(),
                refresh_token: r_token,
                id_token: cl.id_token,
                token_type: cl.token_type,
            }),
            None => Err(FawnError::Google),
        }
    }
}

impl Login {
    pub fn new(code: &str, client_id: &str, client_secret: &str) -> Result<Self, FawnError> {
        return Ok(ureq::post("https://oauth2.googleapis.com/token")
            .send_form([
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", "http://localhost:5003/auth"),
            ])?
            .body_mut()
            .read_json::<GoogleLogin>()?
            .wrap(client_id.to_string(), client_secret.to_string())?);
    }
    pub fn from_refresh_token(
        refresh_token: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<Self, FawnError> {
        let mut login = Login {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            refresh_token: refresh_token.to_string(),
            access_token: String::new(),
            expires: Zoned::now().datetime(),
            id_token: String::new(),
            token_type: String::from("Bearer"),
        };
        login.refresh()?;
        Ok(login)
    }
    pub fn refresh(&mut self) -> Result<(), FawnError> {
        let login = ureq::post("https://oauth2.googleapis.com/token")
            .send_form([
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("refresh_token", &self.refresh_token),
                ("grant_type", &String::from("refresh_token")),
                ("redirect_uri", &String::from("http://localhost:5003/auth")),
            ])?
            .body_mut()
            .read_json::<GoogleLogin>()?;
        self.access_token = login.access_token;
        self.expires = Zoned::now().datetime() + login.expires_in.seconds();
        self.id_token = login.id_token;
        self.token_type = login.token_type;
        return Ok(());
    }
}
