use jiff::{Timestamp, civil::DateTime};

#[derive(Debug)]
pub struct TaskList {
    pub id: String,
    pub link: String,
    pub title: String,
}

#[derive(Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due: Timestamp,
}

#[derive(Debug)]
pub struct Event {
    pub location: String,
    pub title: String,
    pub start: Timestamp,
    pub end: Timestamp,
}

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
