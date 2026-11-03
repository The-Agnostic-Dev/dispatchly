//! Giving a waiting order to a rider.

use crate::{
    services::{orders, riders},
    state::AppState,
};

#[derive(Debug)]
pub enum AssignError {
    NotWaiting(String),
    RiderInactive(String),
    RiderFull { name: String, capacity: i64 },
    Notify(String),
    Store(String),
}

impl std::fmt::Display for AssignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignError::NotWaiting(reference) => write!(f, "{reference} is no longer waiting for a rider."),
            AssignError::RiderInactive(name) => write!(f, "{name} is not on shift."),
            AssignError::RiderFull { name, capacity } => {
                write!(f, "{name} already has {capacity} jobs. Pick another rider or wait for a drop-off.")
            }
            AssignError::Notify(msg) => write!(f, "Could not text the rider: {msg}"),
            AssignError::Store(msg) => write!(f, "Could not save the assignment: {msg}"),
        }
    }
}

impl From<rusqlite::Error> for AssignError {
    fn from(err: rusqlite::Error) -> Self {
        AssignError::Store(err.to_string())
    }
}

impl From<crate::error::AppError> for AssignError {
    fn from(err: crate::error::AppError) -> Self {
        AssignError::Store(format!("{err:?}"))
    }
}

/// Assigns `order_id` to `rider_id`, respecting the rider's job limit, and texts the rider.
/// Returns the rider's name for the confirmation message.
pub async fn assign(state: &AppState, order_id: i64, rider_id: i64, actor: &str) -> Result<String, AssignError> {
    let (order, rider, carrying) = {
        let conn = state.db();
        let order = orders::get(&conn, order_id)?;
        let rider = riders::get(&conn, rider_id)?;
        let carrying = riders::active_jobs(&conn, rider_id)?;
        (order, rider, carrying)
    };

    if order.status != "pending" {
        return Err(AssignError::NotWaiting(order.reference));
    }
    if !rider.active {
        return Err(AssignError::RiderInactive(rider.name));
    }
    if carrying >= rider.capacity {
        return Err(AssignError::RiderFull { name: rider.name, capacity: rider.capacity });
    }

    // Only record the assignment once the rider has actually been told about it.
    state
        .notifier
        .job_offer(&rider.phone, &rider.name, &order.reference, &order.dropoff_area)
        .await
        .map_err(|e| AssignError::Notify(e.0))?;

    let conn = state.db();
    conn.execute(
        "UPDATE orders SET rider_id = ?1, status = 'assigned', assigned_at = ?2 WHERE id = ?3",
        rusqlite::params![rider.id, crate::clock::to_db(crate::clock::now()), order.id],
    )?;
    orders::log_event(&conn, order.id, "assigned", &format!("Assigned to {}", rider.name), actor)?;
    Ok(rider.name)
}
