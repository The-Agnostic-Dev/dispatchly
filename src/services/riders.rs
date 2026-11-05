use std::cmp::Reverse;

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use serde::Serialize;

use crate::{
    clock,
    services::orders::{ORDER_COLUMNS, ORDER_FROM, Order},
};

#[derive(Debug, Clone, Serialize)]
pub struct Rider {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub bike_plate: String,
    pub capacity: i64,
    pub active: bool,
}

/// A rider with today's numbers, for the riders page and the assign dropdown.
#[derive(Debug, Clone, Serialize)]
pub struct RiderSummary {
    #[serde(flatten)]
    pub rider: Rider,
    pub active_jobs: i64,
    pub delivered_today: i64,
    pub cash_on_hand_kobo: i64,
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Rider> {
    conn.query_row(
        "SELECT id, name, phone, bike_plate, capacity, active FROM riders WHERE id = ?1",
        [id],
        |r| {
            Ok(Rider {
                id: r.get(0)?,
                name: r.get(1)?,
                phone: r.get(2)?,
                bike_plate: r.get(3)?,
                capacity: r.get(4)?,
                active: r.get(5)?,
            })
        },
    )
}

/// Jobs a rider is currently carrying (assigned or picked up).
pub fn active_jobs(conn: &Connection, rider_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM orders WHERE rider_id = ?1 AND status IN ('assigned', 'picked_up')",
        [rider_id],
        |r| r.get(0),
    )
}

/// Cash collected on delivery that has not yet been handed in to accounts.
pub fn cash_on_hand(conn: &Connection, rider_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT
            (SELECT COALESCE(SUM(cash_collected_kobo), 0) FROM orders WHERE rider_id = ?1 AND status = 'delivered')
          - (SELECT COALESCE(SUM(amount_kobo), 0) FROM remittances WHERE rider_id = ?1)",
        [rider_id],
        |r| r.get(0),
    )
}

pub fn list(conn: &Connection, today: NaiveDate) -> rusqlite::Result<Vec<RiderSummary>> {
    let riders: Vec<i64> = conn
        .prepare("SELECT id FROM riders ORDER BY active DESC, name")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let (start, end) = clock::day_bounds(today);
    riders
        .into_iter()
        .map(|id| {
            let rider = get(conn, id)?;
            let delivered_today = conn.query_row(
                "SELECT COUNT(*) FROM orders WHERE rider_id = ?1 AND status = 'delivered'
                   AND delivered_at >= ?2 AND delivered_at < ?3",
                params![id, start, end],
                |r| r.get(0),
            )?;
            Ok(RiderSummary {
                active_jobs: active_jobs(conn, id)?,
                delivered_today,
                cash_on_hand_kobo: cash_on_hand(conn, id)?,
                rider,
            })
        })
        .collect()
}

/// The riders page: who is holding the most cash, first.
pub fn list_by_cash(conn: &Connection) -> rusqlite::Result<Vec<RiderSummary>> {
    let mut fleet = list(conn, clock::today())?;
    fleet.sort_by_cached_key(|r| Reverse(r.cash_on_hand_kobo));
    Ok(fleet)
}

/// Current jobs first (oldest assignment first), then the last few finished ones.
pub fn jobs(conn: &Connection, rider_id: i64) -> rusqlite::Result<(Vec<Order>, Vec<Order>)> {
    let current_sql = format!(
        "SELECT {ORDER_COLUMNS} {ORDER_FROM}
         WHERE o.rider_id = ?1 AND o.status IN ('assigned', 'picked_up') ORDER BY o.assigned_at"
    );
    let current = conn.prepare(&current_sql)?.query_map([rider_id], Order::from_row)?.collect::<Result<_, _>>()?;
    let recent_sql = format!(
        "SELECT {ORDER_COLUMNS} {ORDER_FROM}
         WHERE o.rider_id = ?1 AND o.status IN ('delivered', 'failed') ORDER BY o.delivered_at DESC LIMIT 10"
    );
    let recent = conn.prepare(&recent_sql)?.query_map([rider_id], Order::from_row)?.collect::<Result<_, _>>()?;
    Ok((current, recent))
}
