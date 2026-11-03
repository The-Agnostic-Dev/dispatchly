use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, params};

use super::catalog::{MERCHANTS, RIDERS};
use crate::{clock, services::pricing::Tariff};

/// One seeded order. Assignment, pickup and drop-off follow creation at fixed intervals.
pub(super) struct Spec {
    pub merchant: usize,
    pub customer: &'static str,
    pub phone: &'static str,
    pub area: &'static str,
    pub address: &'static str,
    pub km: f64,
    pub express: bool,
    pub cod: i64,
    pub created: DateTime<Utc>,
    pub rider: Option<i64>,
    pub status: &'static str,
}

pub(super) fn insert(conn: &Connection, tariff: &Tariff, spec: &Spec) -> rusqlite::Result<()> {
    let (_, merchant_phone, pickup_area, pickup_address) = MERCHANTS[spec.merchant];
    let area_id = |name: &str| tariff.areas.iter().find(|a| a.name == name).map(|a| a.id).expect("seeded area");
    let (pickup, dropoff) = (area_id(pickup_area), area_id(spec.area));
    let quote = tariff.quote(pickup, dropoff, spec.km, spec.express).expect("seeded quote");
    let assigned = spec.rider.map(|_| spec.created + Duration::minutes(12));
    let picked = spec.created + Duration::minutes(25);
    let finished = spec.created + Duration::minutes(50);
    let done = matches!(spec.status, "delivered" | "failed");
    let collected = if spec.status == "delivered" { spec.cod } else { 0 };

    conn.execute(
        "INSERT INTO orders (merchant_name, merchant_phone, pickup_area_id, pickup_address, customer_name, customer_phone,
            dropoff_area_id, dropoff_address, distance_km, express, cod_kobo, zone_fee_kobo, band_fee_kobo, bridge_fee_kobo,
            express_fee_kobo, fee_kobo, status, rider_id, created_at, assigned_at, delivered_at, cash_collected_kobo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            MERCHANTS[spec.merchant].0, merchant_phone, pickup, pickup_address, spec.customer, spec.phone, dropoff,
            spec.address, spec.km, spec.express, spec.cod, quote.zone_fee_kobo, quote.band_fee_kobo,
            quote.bridge_fee_kobo, quote.express_fee_kobo, quote.total_kobo, spec.status, spec.rider,
            clock::to_db(spec.created), assigned.map(clock::to_db), done.then(|| clock::to_db(finished)), collected
        ],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute("UPDATE orders SET reference = ?1 WHERE id = ?2", params![crate::services::orders::reference_for(id), id])?;

    let rider_name = spec.rider.map(|r| RIDERS[(r - 1) as usize].0).unwrap_or_default();
    let mut events = vec![("pending", "Order received".to_string(), "Tolulope Adebayo", spec.created)];
    if let Some(at) = assigned {
        events.push(("assigned", format!("Assigned to {rider_name}"), "Tolulope Adebayo", at));
    }
    if matches!(spec.status, "picked_up" | "delivered" | "failed") {
        events.push(("picked_up", format!("Picked up from {}", MERCHANTS[spec.merchant].0), rider_name, picked));
    }
    match spec.status {
        "delivered" if spec.cod > 0 => events.push(("delivered", format!("Delivered, collected {}", crate::money::format_naira(spec.cod)), rider_name, finished)),
        "delivered" => events.push(("delivered", "Delivered (prepaid)".into(), rider_name, finished)),
        "failed" => events.push(("failed", "Customer not reachable, parcel returned".into(), rider_name, finished)),
        "cancelled" => events.push(("cancelled", "Merchant cancelled".into(), "Ngozi Eze", spec.created + Duration::minutes(20))),
        _ => {}
    }
    for (status, note, actor, at) in events {
        conn.execute(
            "INSERT INTO order_events (order_id, status, note, actor, at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, status, note, actor, clock::to_db(at)],
        )?;
    }
    Ok(())
}
