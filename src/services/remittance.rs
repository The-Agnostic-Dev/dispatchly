//! End-of-day cash: what each rider collected on cash-on-delivery orders and what they handed in.
//! Delivery fees are invoiced to merchants weekly; riders only carry the goods value.

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use serde::Serialize;

use crate::clock;

#[derive(Debug, Clone, Serialize)]
pub struct SheetRow {
    pub rider_id: i64,
    pub rider_name: String,
    pub cash_deliveries: i64,
    pub expected_kobo: i64,
    pub received_kobo: i64,
    pub outstanding_kobo: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Line {
    pub order_id: i64,
    pub reference: String,
    pub customer_name: String,
    pub dropoff_area: String,
    pub delivered_at: String,
    pub delivered_local: String,
    pub cod_kobo: i64,
    pub collected_kobo: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Remittance {
    pub amount_kobo: i64,
    pub received_by: String,
    pub note: String,
    pub at: String,
}

/// One row per rider who collected cash or handed some in on `day`.
pub fn day_sheet(conn: &Connection, day: NaiveDate) -> rusqlite::Result<Vec<SheetRow>> {
    let (start, end) = clock::day_bounds(day);
    let mut stmt = conn.prepare(
        "SELECT r.id, r.name,
                (SELECT COUNT(*) FROM orders o WHERE o.rider_id = r.id AND o.status = 'delivered'
                   AND o.cod_kobo > 0 AND o.delivered_at >= ?1 AND o.delivered_at < ?2),
                (SELECT COALESCE(SUM(o.cash_collected_kobo), 0) FROM orders o WHERE o.rider_id = r.id
                   AND o.status = 'delivered' AND o.cod_kobo > 0 AND o.delivered_at >= ?1 AND o.delivered_at < ?2),
                (SELECT COALESCE(SUM(m.amount_kobo), 0) FROM remittances m WHERE m.rider_id = r.id AND m.day = ?3)
         FROM riders r ORDER BY r.name",
    )?;
    let rows = stmt.query_map(params![start, end, day.to_string()], |r| {
        let expected: i64 = r.get(3)?;
        let received: i64 = r.get(4)?;
        Ok(SheetRow {
            rider_id: r.get(0)?,
            rider_name: r.get(1)?,
            cash_deliveries: r.get(2)?,
            expected_kobo: expected,
            received_kobo: received,
            outstanding_kobo: expected - received,
        })
    })?;
    Ok(rows
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|row| row.cash_deliveries > 0 || row.received_kobo > 0)
        .collect())
}

/// The cash-on-delivery orders behind a rider's expected amount for `day`.
pub fn rider_lines(conn: &Connection, rider_id: i64, day: NaiveDate) -> rusqlite::Result<Vec<Line>> {
    let (start, end) = clock::day_bounds(day);
    conn.prepare(
        "SELECT o.id, o.reference, o.customer_name, a.name, o.delivered_at, o.cod_kobo, o.cash_collected_kobo
         FROM orders o JOIN areas a ON a.id = o.dropoff_area_id
         WHERE o.rider_id = ?1 AND o.status = 'delivered' AND o.cod_kobo > 0
           AND o.delivered_at >= ?2 AND o.delivered_at < ?3
         ORDER BY o.delivered_at",
    )?
    .query_map(params![rider_id, start, end], |r| {
        let delivered_at: String = r.get(4)?;
        Ok(Line {
            order_id: r.get(0)?,
            reference: r.get(1)?,
            customer_name: r.get(2)?,
            dropoff_area: r.get(3)?,
            delivered_local: clock::local_iso(&delivered_at),
            delivered_at,
            cod_kobo: r.get(5)?,
            collected_kobo: r.get(6)?,
        })
    })?
    .collect()
}

pub fn handed_in(conn: &Connection, rider_id: i64, day: NaiveDate) -> rusqlite::Result<Vec<Remittance>> {
    conn.prepare("SELECT amount_kobo, received_by, note, at FROM remittances WHERE rider_id = ?1 AND day = ?2 ORDER BY at")?
        .query_map(params![rider_id, day.to_string()], |r| {
            Ok(Remittance { amount_kobo: r.get(0)?, received_by: r.get(1)?, note: r.get(2)?, at: r.get(3)? })
        })?
        .collect()
}

pub fn record(conn: &Connection, rider_id: i64, day: NaiveDate, amount_kobo: i64, received_by: &str, note: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO remittances (rider_id, day, amount_kobo, received_by, note, at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![rider_id, day.to_string(), amount_kobo, received_by, note, clock::to_db(clock::now())],
    )?;
    Ok(())
}
