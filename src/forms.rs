//! Form payloads and their validation.

use serde::{Deserialize, Serialize};

use crate::{
    money,
    services::{orders::{Order, OrderInput}, pricing::Tariff},
    validate::{clean_text, normalize_phone, parse_distance},
};

/// The new-order / edit-order form, kept as strings so it can be re-shown with errors.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct OrderForm {
    pub merchant_name: String,
    pub merchant_phone: String,
    pub pickup_area_id: String,
    pub pickup_address: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub dropoff_area_id: String,
    pub dropoff_address: String,
    pub distance_km: String,
    pub cod: String,
    pub express: Option<String>,
    pub notes: String,
}

impl OrderForm {
    pub fn from_order(order: &Order) -> Self {
        Self {
            merchant_name: order.merchant_name.clone(),
            merchant_phone: order.merchant_phone.clone(),
            pickup_area_id: order.pickup_area_id.to_string(),
            pickup_address: order.pickup_address.clone(),
            customer_name: order.customer_name.clone(),
            customer_phone: order.customer_phone.clone(),
            dropoff_area_id: order.dropoff_area_id.to_string(),
            dropoff_address: order.dropoff_address.clone(),
            distance_km: format!("{}", order.distance_km),
            cod: if order.cod_kobo > 0 { money::plain_naira(order.cod_kobo) } else { String::new() },
            express: order.express.then(|| "on".to_string()),
            notes: order.notes.clone(),
        }
    }

    pub fn validate(&self, tariff: &Tariff) -> Result<OrderInput, Vec<String>> {
        let mut errors = Vec::new();
        let mut required = |value: &str, label: &str| {
            clean_text(value).unwrap_or_else(|| {
                errors.push(format!("{label} is required."));
                String::new()
            })
        };
        let merchant_name = required(&self.merchant_name, "Merchant name");
        let pickup_address = required(&self.pickup_address, "Pickup address");
        let customer_name = required(&self.customer_name, "Customer name");
        let dropoff_address = required(&self.dropoff_address, "Drop-off address");

        let merchant_phone = normalize_phone(&self.merchant_phone).unwrap_or_else(|| {
            errors.push("Merchant phone must be a Nigerian mobile number, like 0803 123 4567.".into());
            String::new()
        });
        let customer_phone = normalize_phone(&self.customer_phone).unwrap_or_else(|| {
            errors.push("Customer phone must be a Nigerian mobile number, like 0803 123 4567.".into());
            String::new()
        });

        let area = |value: &str, label: &str, errors: &mut Vec<String>| {
            value
                .parse::<i64>()
                .ok()
                .filter(|id| tariff.area(*id).is_some())
                .unwrap_or_else(|| {
                    errors.push(format!("Choose a {label} area."));
                    0
                })
        };
        let pickup_area_id = area(&self.pickup_area_id, "pickup", &mut errors);
        let dropoff_area_id = area(&self.dropoff_area_id, "drop-off", &mut errors);

        let distance_km = parse_distance(&self.distance_km).unwrap_or_else(|| {
            errors.push("Distance must be between 0.1 and 80 km.".into());
            0.0
        });
        let cod_kobo = money::parse_naira(&self.cod).unwrap_or_else(|| {
            errors.push("Cash to collect must be an amount in naira, like 12500.".into());
            0
        });

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(OrderInput {
            merchant_name,
            merchant_phone,
            pickup_area_id,
            pickup_address,
            customer_name,
            customer_phone,
            dropoff_area_id,
            dropoff_address,
            distance_km,
            express: self.express.is_some(),
            cod_kobo,
            notes: self.notes.trim().to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct QuoteQuery {
    #[serde(default)]
    pub pickup_area_id: String,
    #[serde(default)]
    pub dropoff_area_id: String,
    #[serde(default)]
    pub distance_km: String,
    pub express: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignForm {
    pub rider_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct StatusForm {
    pub action: String,
    #[serde(default)]
    pub cash: String,
    #[serde(default)]
    pub back: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RemitForm {
    pub day: String,
    pub amount: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct DayQuery {
    pub date: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
}
