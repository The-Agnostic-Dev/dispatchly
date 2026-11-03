//! The dispatch board: what is waiting, what is on the road, what landed today.

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::Serialize;

use crate::{
    clock,
    services::orders::{ORDER_COLUMNS, ORDER_FROM, Order},
};

#[derive(Debug, Serialize)]
pub struct Board {
    pub waiting: Vec<Order>,
    pub on_the_road: Vec<Order>,
    pub delivered_today: Vec<Order>,
    pub query: String,
}

pub fn load(conn: &Connection, today: NaiveDate, query: &str) -> rusqlite::Result<Board> {
    let waiting = select(conn, "o.status = 'pending' ORDER BY o.created_at DESC", &[])?;
    let on_the_road = select(
        conn,
        "o.status IN ('assigned', 'picked_up') ORDER BY o.assigned_at",
        &[],
    )?;
    let (start, end) = clock::day_bounds(today);
    let delivered_today = select(
        conn,
        "o.status = 'delivered' AND o.delivered_at >= ?1 AND o.delivered_at < ?2 ORDER BY o.delivered_at DESC",
        &[&start, &end],
    )?;

    let query = query.trim().to_string();
    let keep = |orders: Vec<Order>| -> Vec<Order> {
        if query.is_empty() {
            orders
        } else {
            orders.into_iter().filter(|o| matches(o, &query)).collect()
        }
    };
    Ok(Board {
        waiting: keep(waiting),
        on_the_road: keep(on_the_road),
        delivered_today: keep(delivered_today),
        query,
    })
}

/// Search box: order reference, customer, phone, merchant or drop-off area.
fn matches(order: &Order, query: &str) -> bool {
    [
        &order.reference,
        &order.customer_name,
        &order.customer_phone,
        &order.merchant_name,
        &order.dropoff_area,
    ]
    .iter()
    .any(|field| field.contains(query))
}

fn select(conn: &Connection, filter: &str, args: &[&dyn rusqlite::ToSql]) -> rusqlite::Result<Vec<Order>> {
    let sql = format!("SELECT {ORDER_COLUMNS} {ORDER_FROM} WHERE {filter}");
    conn.prepare(&sql)?.query_map(args, Order::from_row)?.collect()
}
