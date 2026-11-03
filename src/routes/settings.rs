use std::collections::HashMap;

use axum::{
    Form,
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use minijinja::context;

use crate::{
    auth::CurrentUser,
    error::AppResult,
    money,
    services::pricing::{self, BRIDGE_FEE_KOBO, EXPRESS_FEE_KOBO, Tariff},
    state::AppState,
    views,
};

pub async fn tariff(State(state): State<AppState>, user: CurrentUser) -> AppResult<Response> {
    user.require(user.role.can_dispatch())?;
    let tariff = {
        let conn = state.db();
        Tariff::load(&conn)?
    };
    let page = views::render(
        &state,
        Some(&user),
        "tariff.html",
        context! {
            tariff => tariff,
            bridge_fee_kobo => BRIDGE_FEE_KOBO,
            express_fee_kobo => EXPRESS_FEE_KOBO,
            can_edit => user.role.can_edit_tariff(),
        },
    )?;
    Ok(page.into_response())
}

/// Reads `prefix_<id>` fields as naira amounts. Returns None if any amount is unreadable.
fn fees(form: &HashMap<String, String>, prefix: &str) -> Option<Vec<(i64, i64)>> {
    form.iter()
        .filter_map(|(key, value)| Some((key.strip_prefix(prefix)?.parse::<i64>().ok()?, value)))
        .map(|(id, value)| money::parse_naira(value).map(|kobo| (id, kobo)))
        .collect()
}

pub async fn save_zones(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(form): Form<HashMap<String, String>>,
) -> AppResult<Response> {
    user.require(user.role.can_edit_tariff())?;
    let Some(fees) = fees(&form, "zone_") else {
        state.sessions.flash(&user.token, "error", "Zone fees must be amounts in naira.");
        return Ok(Redirect::to("/settings/tariff").into_response());
    };
    {
        let conn = state.db();
        for (zone_id, kobo) in fees {
            pricing::update_zone_fee(&conn, zone_id, kobo)?;
        }
    }
    state.tariff.invalidate();
    state.sessions.flash(&user.token, "success", "Zone fees saved. New orders use them from now on.");
    Ok(Redirect::to("/settings/tariff").into_response())
}

pub async fn save_bands(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(form): Form<HashMap<String, String>>,
) -> AppResult<Response> {
    user.require(user.role.can_edit_tariff())?;
    let Some(fees) = fees(&form, "band_") else {
        state.sessions.flash(&user.token, "error", "Distance band fees must be amounts in naira.");
        return Ok(Redirect::to("/settings/tariff").into_response());
    };
    {
        let conn = state.db();
        for (band_id, kobo) in fees {
            pricing::update_band_fee(&conn, band_id, kobo)?;
        }
    }
    state.sessions.flash(&user.token, "success", "Distance band fees saved. New orders use them from now on.");
    Ok(Redirect::to("/settings/tariff").into_response())
}
