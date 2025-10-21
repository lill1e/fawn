mod structs;

use jiff::{ToSpan, Zoned, tz::TimeZone};
pub use structs::{fawn::*, google::*};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum FawnError {
    #[error("request")]
    Request(#[from] ureq::Error),
    #[error("json")]
    JSON(#[from] serde_json::Error),
    #[error("url")]
    URLBuilder(#[from] url::ParseError),
    #[error("date")]
    Date(#[from] jiff::Error),
    #[error("google login")]
    Google,
    #[error("system timezone")]
    SysTz,
}

impl From<()> for FawnError {
    fn from(_: ()) -> Self {
        FawnError::URLBuilder(url::ParseError::InvalidDomainCharacter)
    }
}

impl GoogleTasklists {
    pub fn format(&self) -> Vec<TaskList> {
        return self
            .items
            .iter()
            .map(|item| TaskList {
                id: item.id.clone(),
                link: item.self_link.clone(),
                title: item.title.clone(),
            })
            .collect();
    }
}

impl GoogleCalendarlists {
    pub fn format(&self) -> Vec<CalendarList> {
        return self
            .items
            .iter()
            .map(|item| CalendarList {
                id: item.id.clone(),
                title: item.summary.clone(),
            })
            .collect();
    }
}

impl GoogleCalendarEvents {
    pub fn format(&self) -> Vec<Event> {
        return self
            .items
            .iter()
            .map(|item| Event {
                location: item.location.clone(),
                title: item.summary.clone(),
                start: item.start.date_time.parse().unwrap_or_default(),
                end: item.end.date_time.parse().unwrap_or_default(),
            })
            .collect();
    }
}

impl TaskList {
    pub fn tasks(&self, login: &Login) -> Result<Vec<Task>, FawnError> {
        // TODO: add options for filtering
        Ok(ureq::get(format!(
            "https://tasks.googleapis.com/tasks/v1/lists/{}/tasks",
            &self.id
        ))
        .header("Authorization", format!("Bearer {}", login.access_token))
        .call()?
        .body_mut()
        .read_json::<GoogleTaskslist>()?
        .items
        .into_iter()
        .map(|t| Task {
            id: t.id,
            title: t.title,
            description: t.notes,
            due: t.due.parse().unwrap_or_default(),
        })
        .collect())
    }
}

impl CalendarList {
    pub fn events(&self, login: &Login) -> Result<GoogleCalendarEvents, FawnError> {
        let mut url = Url::parse("https://www.googleapis.com/calendar/v3/calendars")?;
        url.path_segments_mut()?.extend(&[&self.id, "events"]);
        let today = Zoned::now();
        return Ok(ureq::get(url.to_string())
            .query("singleEvents", "true")
            .query("orderBy", "startTime")
            .query("timeMin", today.timestamp().to_string())
            .query(
                "timeMax",
                today
                    .tomorrow()
                    .unwrap_or_default()
                    .start_of_day()
                    .unwrap_or_default()
                    .timestamp()
                    .to_string(),
            )
            .query(
                "timeZone",
                TimeZone::system().iana_name().ok_or(FawnError::SysTz)?,
            )
            .header("Authorization", format!("Bearer {}", login.access_token))
            .call()?
            .body_mut()
            .read_json::<GoogleCalendarEvents>()?);
    }
}

impl GoogleLogin {
    fn wrap(&self, client_id: String, client_secret: String) -> Result<Login, FawnError> {
        let cl = self.clone();
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

    pub fn tasklist(&self) -> Result<Vec<TaskList>, FawnError> {
        Ok(
            ureq::get("https://tasks.googleapis.com/tasks/v1/users/@me/lists")
                .header("Authorization", format!("Bearer {}", self.access_token))
                .call()?
                .body_mut()
                .read_json::<GoogleTasklists>()?
                .format(),
        )
    }

    pub fn all_tasks(&self) -> Result<Vec<Task>, FawnError> {
        Ok(self
            .tasklist()?
            .iter()
            .flat_map(|list| list.tasks(&self).unwrap_or(vec![]))
            .collect::<Vec<Task>>())
    }
    pub fn calendarlist(&self) -> Result<Vec<CalendarList>, FawnError> {
        Ok(
            ureq::get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
                .header("Authorization", format!("Bearer {}", self.access_token))
                .call()?
                .body_mut()
                .read_json::<GoogleCalendarlists>()?
                .format(),
        )
    }

    pub fn all_events(&self) -> Result<Vec<Event>, FawnError> {
        Ok(self
            .calendarlist()?
            .iter()
            .flat_map(|l| {
                l.events(&self)
                    .unwrap_or(GoogleCalendarEvents { items: Vec::new() })
                    .format()
            })
            .collect::<Vec<_>>())
    }
}
