//! Logins and in-memory sessions (staff sign in again after a restart).

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::state::AppState;

pub const SESSION_COOKIE: &str = "dispatchly_session";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Dispatcher,
    OpsLead,
    Accountant,
    Rider,
}

impl Role {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "dispatcher" => Some(Role::Dispatcher),
            "ops_lead" => Some(Role::OpsLead),
            "accountant" => Some(Role::Accountant),
            "rider" => Some(Role::Rider),
            _ => None,
        }
    }

    pub fn can_dispatch(self) -> bool {
        matches!(self, Role::Dispatcher | Role::OpsLead)
    }

    pub fn can_see_money(self) -> bool {
        matches!(self, Role::Accountant | Role::OpsLead)
    }

    pub fn can_edit_tariff(self) -> bool {
        self == Role::OpsLead
    }

    pub fn home(self) -> &'static str {
        match self {
            Role::Rider => "/my-jobs",
            Role::Accountant => "/remittance",
            _ => "/",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub rider_id: Option<i64>,
    #[serde(skip)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Flash {
    pub kind: &'static str,
    pub message: String,
}

#[derive(Debug, Clone)]
struct Session {
    user: CurrentUser,
    flash: Option<Flash>,
}

#[derive(Clone, Default)]
pub struct SessionStore(Arc<Mutex<HashMap<String, Session>>>);

impl SessionStore {
    pub fn start(&self, mut user: CurrentUser) -> String {
        let token = new_token(&user.username);
        user.token = token.clone();
        self.lock().insert(token.clone(), Session { user, flash: None });
        token
    }

    pub fn user(&self, token: &str) -> Option<CurrentUser> {
        self.lock().get(token).map(|s| s.user.clone())
    }

    pub fn end(&self, token: &str) {
        self.lock().remove(token);
    }

    pub fn flash(&self, token: &str, kind: &'static str, message: impl Into<String>) {
        if let Some(session) = self.lock().get_mut(token) {
            session.flash = Some(Flash { kind, message: message.into() });
        }
    }

    pub fn take_flash(&self, token: &str) -> Option<Flash> {
        self.lock().get_mut(token).and_then(|s| s.flash.take())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Session>> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }
}

impl CurrentUser {
    pub fn require(&self, allowed: bool) -> Result<(), crate::error::AppError> {
        if allowed { Ok(()) } else { Err(crate::error::AppError::Forbidden) }
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        cookie(&parts.headers, SESSION_COOKIE)
            .and_then(|token| state.sessions.user(&token))
            .ok_or_else(|| Redirect::to("/login").into_response())
    }
}

pub fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v.to_string())
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let digest = Sha256::digest(format!("{salt}:{password}").as_bytes());
    format!("{salt}${}", hex(&digest))
}

fn verify_password(password: &str, stored: &str) -> bool {
    stored.split_once('$').is_some_and(|(salt, _)| hash_password(password, salt) == stored)
}

/// Checks a username and password against the users table.
pub fn authenticate(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<Option<CurrentUser>> {
    let row = conn
        .query_row(
            "SELECT id, username, display_name, role, password_hash, rider_id FROM users WHERE username = ?1",
            params![username.trim().to_lowercase()],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<i64>>(5)?,
                ))
            },
        )
        .optional()?;
    Ok(row.and_then(|(id, username, display_name, role, hash, rider_id)| {
        let role = Role::parse(&role)?;
        verify_password(password, &hash).then(|| CurrentUser {
            id,
            username,
            display_name,
            role,
            rider_id,
            token: String::new(),
        })
    }))
}

fn new_token(seed: &str) -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write(seed.as_bytes());
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let digest = Sha256::digest(format!("{}:{nanos}:{seed}", hasher.finish()).as_bytes());
    hex(&digest)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
