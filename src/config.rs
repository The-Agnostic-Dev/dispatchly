use std::{env, path::PathBuf, time::Duration};

/// Runtime settings, read from the environment (see `.env.example`).
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub db_path: String,
    pub templates_dir: PathBuf,
    pub static_dir: PathBuf,
    pub reseed: bool,
    pub sms_latency: Duration,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5115);
        let db_path = env::var("DISPATCHLY_DB").unwrap_or_else(|_| "dispatchly.sqlite3".into());
        let templates_dir = env::var("DISPATCHLY_TEMPLATES").unwrap_or_else(|_| "templates".into());
        let static_dir = env::var("DISPATCHLY_STATIC").unwrap_or_else(|_| "static".into());
        let reseed = matches!(env::var("DISPATCHLY_RESEED").as_deref(), Ok("1") | Ok("true"));
        let sms_latency = env::var("DISPATCHLY_SMS_LATENCY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(300));
        Self {
            port,
            db_path,
            templates_dir: templates_dir.into(),
            static_dir: static_dir.into(),
            reseed,
            sms_latency,
        }
    }
}
