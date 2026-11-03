use std::sync::{Arc, Mutex, MutexGuard};

use minijinja::Environment;
use rusqlite::Connection;

use crate::{auth::SessionStore, config::Config, notify::Notifier, services::pricing::TariffCache, views};

#[derive(Clone)]
pub struct AppState {
    db: Arc<Mutex<Connection>>,
    pub templates: Arc<Environment<'static>>,
    pub sessions: SessionStore,
    pub tariff: TariffCache,
    pub notifier: Notifier,
}

impl AppState {
    pub fn new(conn: Connection, config: &Config) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            templates: Arc::new(views::environment(&config.templates_dir)),
            sessions: SessionStore::default(),
            tariff: TariffCache::default(),
            notifier: Notifier::new(config.sms_latency),
        }
    }

    /// Locks the shared connection. Keep the guard short-lived and never hold it across an `.await`.
    pub fn db(&self) -> MutexGuard<'_, Connection> {
        self.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
