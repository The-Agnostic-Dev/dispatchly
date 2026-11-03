//! Delivery fee = drop-off zone base fee + distance band fee
//!              + bridge fee when the trip crosses between Mainland and Island
//!              + express fee when the merchant asks for express.

use std::sync::{Arc, RwLock};

use rusqlite::Connection;
use serde::Serialize;

pub const BRIDGE_FEE_KOBO: i64 = 50_000;
pub const EXPRESS_FEE_KOBO: i64 = 100_000;

#[derive(Debug, Clone, Serialize)]
pub struct Zone {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub side: String,
    pub base_fee_kobo: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Area {
    pub id: i64,
    pub name: String,
    pub zone_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Band {
    pub id: i64,
    pub label: String,
    pub min_km: f64,
    pub max_km: Option<f64>,
    pub fee_kobo: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tariff {
    pub zones: Vec<Zone>,
    pub areas: Vec<Area>,
    pub bands: Vec<Band>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Quote {
    pub zone_name: String,
    pub band_label: String,
    pub zone_fee_kobo: i64,
    pub band_fee_kobo: i64,
    pub bridge_fee_kobo: i64,
    pub express_fee_kobo: i64,
    pub total_kobo: i64,
}

#[derive(Debug, PartialEq)]
pub enum QuoteError {
    UnknownArea,
    NoBand,
}

impl std::fmt::Display for QuoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteError::UnknownArea => write!(f, "Pick a pickup and drop-off area from the list."),
            QuoteError::NoBand => write!(f, "No distance band covers that distance."),
        }
    }
}

impl Tariff {
    pub fn load(conn: &Connection) -> rusqlite::Result<Self> {
        let zones = conn
            .prepare("SELECT id, code, name, side, base_fee_kobo FROM zones ORDER BY code")?
            .query_map([], |r| {
                Ok(Zone { id: r.get(0)?, code: r.get(1)?, name: r.get(2)?, side: r.get(3)?, base_fee_kobo: r.get(4)? })
            })?
            .collect::<Result<_, _>>()?;
        let areas = conn
            .prepare("SELECT id, name, zone_id FROM areas ORDER BY name")?
            .query_map([], |r| Ok(Area { id: r.get(0)?, name: r.get(1)?, zone_id: r.get(2)? }))?
            .collect::<Result<_, _>>()?;
        let bands = conn
            .prepare("SELECT id, label, min_km, max_km, fee_kobo FROM distance_bands ORDER BY min_km")?
            .query_map([], |r| {
                Ok(Band { id: r.get(0)?, label: r.get(1)?, min_km: r.get(2)?, max_km: r.get(3)?, fee_kobo: r.get(4)? })
            })?
            .collect::<Result<_, _>>()?;
        Ok(Self { zones, areas, bands })
    }

    pub fn area(&self, id: i64) -> Option<&Area> {
        self.areas.iter().find(|a| a.id == id)
    }

    pub fn zone_of(&self, area_id: i64) -> Option<&Zone> {
        let area = self.area(area_id)?;
        self.zones.iter().find(|z| z.id == area.zone_id)
    }

    /// The band whose range contains `km` (lower bound inclusive, upper bound exclusive).
    pub fn band_for(&self, km: f64) -> Option<&Band> {
        self.bands.iter().find(|b| km >= b.min_km && b.max_km.is_none_or(|max| km < max))
    }

    pub fn quote(&self, pickup_area: i64, dropoff_area: i64, km: f64, express: bool) -> Result<Quote, QuoteError> {
        let pickup_zone = self.zone_of(pickup_area).ok_or(QuoteError::UnknownArea)?;
        let dropoff_zone = self.zone_of(dropoff_area).ok_or(QuoteError::UnknownArea)?;
        let band = self.band_for(km).ok_or(QuoteError::NoBand)?;
        let bridge = if pickup_zone.side != dropoff_zone.side { BRIDGE_FEE_KOBO } else { 0 };
        let express_fee = if express { EXPRESS_FEE_KOBO } else { 0 };
        Ok(Quote {
            zone_name: dropoff_zone.name.clone(),
            band_label: band.label.clone(),
            zone_fee_kobo: dropoff_zone.base_fee_kobo,
            band_fee_kobo: band.fee_kobo,
            bridge_fee_kobo: bridge,
            express_fee_kobo: express_fee,
            total_kobo: dropoff_zone.base_fee_kobo + band.fee_kobo + bridge + express_fee,
        })
    }
}

/// Keeps the tariff in memory; the quote endpoint is hit on every keystroke of the order form.
#[derive(Clone, Default)]
pub struct TariffCache(Arc<RwLock<Option<Arc<Tariff>>>>);

impl TariffCache {
    pub fn get(&self, conn: &Connection) -> rusqlite::Result<Arc<Tariff>> {
        if let Some(tariff) = self.0.read().unwrap_or_else(|p| p.into_inner()).as_ref() {
            return Ok(Arc::clone(tariff));
        }
        let fresh = Arc::new(Tariff::load(conn)?);
        *self.0.write().unwrap_or_else(|p| p.into_inner()) = Some(Arc::clone(&fresh));
        Ok(fresh)
    }

    pub fn invalidate(&self) {
        *self.0.write().unwrap_or_else(|p| p.into_inner()) = None;
    }
}

pub fn update_zone_fee(conn: &Connection, zone_id: i64, fee_kobo: i64) -> rusqlite::Result<usize> {
    conn.execute("UPDATE zones SET base_fee_kobo = ?1 WHERE id = ?2", (fee_kobo, zone_id))
}

pub fn update_band_fee(conn: &Connection, band_id: i64, fee_kobo: i64) -> rusqlite::Result<usize> {
    conn.execute("UPDATE distance_bands SET fee_kobo = ?1 WHERE id = ?2", (fee_kobo, band_id))
}
