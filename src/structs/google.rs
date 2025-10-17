use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleTasklists {
    pub items: Vec<GoogleTasklist>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleTasklist {
    pub id: String,
    pub self_link: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleTaskslist {
    pub items: Vec<GoogleTasks>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleTasks {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub due: String,
}

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
