#![allow(dead_code)]

use std::time::Duration;

use dispatchly::{config::Config, db, seed, services::pricing::Tariff, state::AppState};
use rusqlite::Connection;

pub fn seeded() -> Connection {
    let conn = db::open_in_memory().expect("in-memory db");
    seed::run(&conn).expect("seed");
    conn
}

pub fn tariff(conn: &Connection) -> Tariff {
    Tariff::load(conn).expect("tariff")
}

pub fn area(tariff: &Tariff, name: &str) -> i64 {
    tariff.areas.iter().find(|a| a.name == name).map(|a| a.id).expect("area")
}

pub fn state() -> AppState {
    let config = Config {
        port: 0,
        db_path: ":memory:".into(),
        templates_dir: "templates".into(),
        static_dir: "static".into(),
        reseed: false,
        sms_latency: Duration::from_millis(1),
    };
    AppState::new(seeded(), &config)
}

pub fn rider_id(conn: &Connection, name: &str) -> i64 {
    conn.query_row("SELECT id FROM riders WHERE name = ?1", [name], |r| r.get(0)).expect("rider")
}
