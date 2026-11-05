//! Delivery orders: creation, edits, status changes and the timeline.

use rusqlite::{Connection, Row, params};
use serde::Serialize;

use crate::{
    clock,
    error::{AppError, AppResult},
    services::pricing::Tariff,
};

pub const ORDER_COLUMNS: &str = "o.id, o.reference, o.merchant_name, o.merchant_phone, o.pickup_area_id, pa.name,
    o.pickup_address, o.customer_name, o.customer_phone, o.dropoff_area_id, da.name, o.dropoff_address,
    o.distance_km, o.express, o.cod_kobo, o.zone_fee_kobo, o.band_fee_kobo, o.bridge_fee_kobo,
    o.express_fee_kobo, o.fee_kobo, o.status, o.rider_id, r.name, r.phone, o.notes, o.created_at,
    o.assigned_at, o.delivered_at, o.cash_collected_kobo";

pub const ORDER_FROM: &str = "FROM orders o
    JOIN areas pa ON pa.id = o.pickup_area_id
    JOIN areas da ON da.id = o.dropoff_area_id
    LEFT JOIN riders r ON r.id = o.rider_id";

#[derive(Debug, Clone, Serialize)]
pub struct Order {
    pub id: i64,
    pub reference: String,
    pub merchant_name: String,
    pub merchant_phone: String,
    pub pickup_area_id: i64,
    pub pickup_area: String,
    pub pickup_address: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub dropoff_area_id: i64,
    pub dropoff_area: String,
    pub dropoff_address: String,
    pub distance_km: f64,
    pub express: bool,
    pub cod_kobo: i64,
    pub zone_fee_kobo: i64,
    pub band_fee_kobo: i64,
    pub bridge_fee_kobo: i64,
    pub express_fee_kobo: i64,
    pub fee_kobo: i64,
    pub status: String,
    pub rider_id: Option<i64>,
    pub rider_name: Option<String>,
    pub rider_phone: Option<String>,
    pub notes: String,
    pub created_at: String,
    pub assigned_at: Option<String>,
    pub delivered_at: Option<String>,
    pub cash_collected_kobo: i64,
}

impl Order {
    pub fn from_row(r: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            reference: r.get(1)?,
            merchant_name: r.get(2)?,
            merchant_phone: r.get(3)?,
            pickup_area_id: r.get(4)?,
            pickup_area: r.get(5)?,
            pickup_address: r.get(6)?,
            customer_name: r.get(7)?,
            customer_phone: r.get(8)?,
            dropoff_area_id: r.get(9)?,
            dropoff_area: r.get(10)?,
            dropoff_address: r.get(11)?,
            distance_km: r.get(12)?,
            express: r.get(13)?,
            cod_kobo: r.get(14)?,
            zone_fee_kobo: r.get(15)?,
            band_fee_kobo: r.get(16)?,
            bridge_fee_kobo: r.get(17)?,
            express_fee_kobo: r.get(18)?,
            fee_kobo: r.get(19)?,
            status: r.get(20)?,
            rider_id: r.get(21)?,
            rider_name: r.get(22)?,
            rider_phone: r.get(23)?,
            notes: r.get(24)?,
            created_at: r.get(25)?,
            assigned_at: r.get(26)?,
            delivered_at: r.get(27)?,
            cash_collected_kobo: r.get(28)?,
        })
    }

    /// Details can be corrected until the rider has the parcel.
    pub fn is_editable(&self) -> bool {
        matches!(self.status.as_str(), "pending" | "assigned")
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status.as_str(), "assigned" | "picked_up")
    }
}

/// Validated order details from the new-order and edit forms.
#[derive(Debug, Clone)]
pub struct OrderInput {
    pub merchant_name: String,
    pub merchant_phone: String,
    pub pickup_area_id: i64,
    pub pickup_address: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub dropoff_area_id: i64,
    pub dropoff_address: String,
    pub distance_km: f64,
    pub express: bool,
    pub cod_kobo: i64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub status: String,
    pub note: String,
    pub actor: String,
    pub at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    PickedUp,
    Delivered,
    Failed,
    Cancel,
}

impl Action {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "picked_up" => Some(Action::PickedUp),
            "delivered" => Some(Action::Delivered),
            "failed" => Some(Action::Failed),
            "cancel" => Some(Action::Cancel),
            _ => None,
        }
    }

    pub fn status(self) -> &'static str {
        match self {
            Action::PickedUp => "picked_up",
            Action::Delivered => "delivered",
            Action::Failed => "failed",
            Action::Cancel => "cancelled",
        }
    }

    pub fn past_tense(self) -> &'static str {
        match self {
            Action::PickedUp => "picked up",
            Action::Delivered => "delivered",
            Action::Failed => "failed",
            Action::Cancel => "cancelled",
        }
    }

    fn allowed_from(self, status: &str) -> bool {
        match self {
            Action::PickedUp => status == "assigned",
            Action::Delivered => status == "picked_up",
            Action::Failed => matches!(status, "assigned" | "picked_up"),
            Action::Cancel => matches!(status, "pending" | "assigned"),
        }
    }
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Order> {
    let sql = format!("SELECT {ORDER_COLUMNS} {ORDER_FROM} WHERE o.id = ?1");
    Ok(conn.query_row(&sql, [id], Order::from_row)?)
}

pub fn create(conn: &Connection, tariff: &Tariff, input: &OrderInput, actor: &str) -> AppResult<i64> {
    let quote = tariff
        .quote(input.pickup_area_id, input.dropoff_area_id, input.distance_km, input.express)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let now = clock::to_db(clock::now());
    conn.execute(
        "INSERT INTO orders (merchant_name, merchant_phone, pickup_area_id, pickup_address, customer_name,
            customer_phone, dropoff_area_id, dropoff_address, distance_km, express, cod_kobo, zone_fee_kobo,
            band_fee_kobo, bridge_fee_kobo, express_fee_kobo, fee_kobo, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            input.merchant_name, input.merchant_phone, input.pickup_area_id, input.pickup_address,
            input.customer_name, input.customer_phone, input.dropoff_area_id, input.dropoff_address,
            input.distance_km, input.express, input.cod_kobo, quote.zone_fee_kobo, quote.band_fee_kobo,
            quote.bridge_fee_kobo, quote.express_fee_kobo, quote.total_kobo, input.notes, now
        ],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute("UPDATE orders SET reference = ?1 WHERE id = ?2", params![reference_for(id), id])?;
    log_event(conn, id, "pending", "Order received", actor)?;
    Ok(id)
}

pub fn reference_for(id: i64) -> String {
    format!("DL-{}", 24_000 + id)
}

pub fn update(conn: &Connection, tariff: &Tariff, id: i64, input: &OrderInput, actor: &str) -> AppResult<()> {
    let order = get(conn, id)?;
    if !order.is_editable() {
        return Err(AppError::BadRequest(format!(
            "{} is already {} and can no longer be edited.",
            order.reference,
            crate::views::status_label(order.status)
        )));
    }
    let quote = tariff
        .quote(input.pickup_area_id, input.dropoff_area_id, input.distance_km, input.express)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    conn.execute(
        "UPDATE orders SET merchant_name = ?1, merchant_phone = ?2, pickup_area_id = ?3, pickup_address = ?4,
            customer_name = ?5, customer_phone = ?6, dropoff_area_id = ?7, dropoff_address = ?8,
            distance_km = ?9, express = ?10, cod_kobo = ?11, notes = ?12,
            zone_fee_kobo = ?13, band_fee_kobo = ?14, bridge_fee_kobo = ?15, express_fee_kobo = ?16, fee_kobo = ?17
         WHERE id = ?18",
        params![
            input.merchant_name, input.merchant_phone, input.pickup_area_id, input.pickup_address,
            input.customer_name, input.customer_phone, input.dropoff_area_id, input.dropoff_address,
            input.distance_km, input.express, input.cod_kobo, input.notes,
            quote.zone_fee_kobo, quote.band_fee_kobo, quote.bridge_fee_kobo, quote.express_fee_kobo, quote.total_kobo,
            id
        ],
    )?;
    log_event(conn, id, &order.status, "Order details edited", actor)?;
    Ok(())
}

pub fn repair_fees(conn: &Connection, tariff: &Tariff) -> AppResult<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, pickup_area_id, dropoff_area_id, distance_km, express, zone_fee_kobo, band_fee_kobo, bridge_fee_kobo, express_fee_kobo, fee_kobo FROM orders"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, bool>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, i64>(8)?,
            row.get::<_, i64>(9)?,
        ))
    })?;
    let mut corrected = 0usize;
    for row in rows {
        let (id, pickup_area_id, dropoff_area_id, distance_km, express, zone_fee, band_fee, bridge_fee, express_fee, fee) = row?;
        let quote = tariff
            .quote(pickup_area_id, dropoff_area_id, distance_km, express)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        if zone_fee != quote.zone_fee_kobo
            || band_fee != quote.band_fee_kobo
            || bridge_fee != quote.bridge_fee_kobo
            || express_fee != quote.express_fee_kobo
            || fee != quote.total_kobo
        {
            conn.execute(
                "UPDATE orders SET zone_fee_kobo = ?1, band_fee_kobo = ?2, bridge_fee_kobo = ?3, express_fee_kobo = ?4, fee_kobo = ?5 WHERE id = ?6",
                params![quote.zone_fee_kobo, quote.band_fee_kobo, quote.bridge_fee_kobo, quote.express_fee_kobo, quote.total_kobo, id],
            )?;
            corrected += 1;
        }
    }
    Ok(corrected)
}

/// Moves an order along its lifecycle. Returns false when the order is not in a state
/// where `action` applies (for example delivering an order nobody has picked up).
pub fn transition(conn: &Connection, id: i64, action: Action, cash_kobo: Option<i64>, actor: &str) -> AppResult<bool> {
    let order = get(conn, id)?;
    if !action.allowed_from(&order.status) {
        return Ok(false);
    }
    let now = clock::to_db(clock::now());
    match action {
        Action::Delivered => {
            let collected = cash_kobo.unwrap_or(order.cod_kobo);
            conn.execute(
                "UPDATE orders SET status = 'delivered', delivered_at = ?1, cash_collected_kobo = ?2 WHERE id = ?3",
                params![now, collected, id],
            )?;
        }
        Action::Cancel => {
            conn.execute("UPDATE orders SET status = 'cancelled', rider_id = NULL WHERE id = ?1", [id])?;
        }
        other => {
            conn.execute("UPDATE orders SET status = ?1 WHERE id = ?2", params![other.status(), id])?;
        }
    }
    let note = match action {
        Action::Delivered if order.cod_kobo > 0 => {
            format!("Delivered, collected {}", crate::money::format_naira(cash_kobo.unwrap_or(order.cod_kobo)))
        }
        Action::Delivered => "Delivered (prepaid)".to_string(),
        Action::PickedUp => format!("Picked up from {}", order.merchant_name),
        Action::Failed => "Delivery attempt failed".to_string(),
        Action::Cancel => "Order cancelled".to_string(),
    };
    log_event(conn, id, action.status(), &note, actor)?;
    Ok(true)
}

pub fn log_event(conn: &Connection, order_id: i64, status: &str, note: &str, actor: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO order_events (order_id, status, note, actor, at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![order_id, status, note, actor, clock::to_db(clock::now())],
    )?;
    Ok(())
}

pub fn timeline(conn: &Connection, order_id: i64) -> rusqlite::Result<Vec<Event>> {
    conn.prepare("SELECT status, note, actor, at FROM order_events WHERE order_id = ?1 ORDER BY at, id")?
        .query_map([order_id], |r| Ok(Event { status: r.get(0)?, note: r.get(1)?, actor: r.get(2)?, at: r.get(3)? }))?
        .collect()
}
