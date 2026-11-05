use rusqlite::Connection;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id            INTEGER PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    role          TEXT NOT NULL CHECK (role IN ('dispatcher', 'ops_lead', 'accountant', 'rider')),
    password_hash TEXT NOT NULL,
    rider_id      INTEGER REFERENCES riders(id)
);

CREATE TABLE IF NOT EXISTS zones (
    id            INTEGER PRIMARY KEY,
    code          TEXT NOT NULL UNIQUE,
    name          TEXT NOT NULL,
    side          TEXT NOT NULL CHECK (side IN ('mainland', 'island')),
    base_fee_kobo INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS areas (
    id      INTEGER PRIMARY KEY,
    name    TEXT NOT NULL UNIQUE,
    zone_id INTEGER NOT NULL REFERENCES zones(id)
);

CREATE TABLE IF NOT EXISTS distance_bands (
    id       INTEGER PRIMARY KEY,
    label    TEXT NOT NULL,
    min_km   REAL NOT NULL,
    max_km   REAL,
    fee_kobo INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS riders (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL,
    phone      TEXT NOT NULL,
    bike_plate TEXT NOT NULL,
    capacity   INTEGER NOT NULL DEFAULT 3,
    active     INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS orders (
    id                  INTEGER PRIMARY KEY,
    reference           TEXT NOT NULL DEFAULT '',
    merchant_name       TEXT NOT NULL,
    merchant_phone      TEXT NOT NULL,
    pickup_area_id      INTEGER NOT NULL REFERENCES areas(id),
    pickup_address      TEXT NOT NULL,
    customer_name       TEXT NOT NULL,
    customer_phone      TEXT NOT NULL,
    dropoff_area_id     INTEGER NOT NULL REFERENCES areas(id),
    dropoff_address     TEXT NOT NULL,
    distance_km         REAL NOT NULL,
    express             INTEGER NOT NULL DEFAULT 0,
    cod_kobo            INTEGER NOT NULL DEFAULT 0,
    zone_fee_kobo       INTEGER NOT NULL,
    band_fee_kobo       INTEGER NOT NULL,
    bridge_fee_kobo     INTEGER NOT NULL,
    express_fee_kobo    INTEGER NOT NULL,
    fee_kobo            INTEGER NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending',
    rider_id            INTEGER REFERENCES riders(id),
    notes               TEXT NOT NULL DEFAULT '',
    created_at          TEXT NOT NULL,
    assigned_at         TEXT,
    delivered_at        TEXT,
    cash_collected_kobo INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS orders_rider ON orders(rider_id, status);

CREATE TABLE IF NOT EXISTS order_events (
    id       INTEGER PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES orders(id),
    status   TEXT NOT NULL,
    note     TEXT NOT NULL,
    actor    TEXT NOT NULL,
    at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS remittances (
    id          INTEGER PRIMARY KEY,
    rider_id    INTEGER NOT NULL REFERENCES riders(id),
    day         TEXT NOT NULL,
    amount_kobo INTEGER NOT NULL,
    received_by TEXT NOT NULL,
    note        TEXT NOT NULL DEFAULT '',
    at          TEXT NOT NULL
);
"#;

pub fn open(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    prepare(conn)
}

pub fn open_in_memory() -> rusqlite::Result<Connection> {
    prepare(Connection::open_in_memory()?)
}

fn prepare(conn: Connection) -> rusqlite::Result<Connection> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}
