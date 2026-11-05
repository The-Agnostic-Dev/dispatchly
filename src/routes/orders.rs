use axum::{
    Form, Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use minijinja::context;
use serde_json::json;

use crate::{
    auth::{CurrentUser, Role},
    clock,
    error::{AppError, AppResult},
    forms::{AssignForm, OrderForm, QuoteQuery, StatusForm},
    money,
    services::{
        dispatch,
        orders::{self, Action},
        riders,
    },
    state::AppState,
    validate::parse_distance,
    views,
};

pub async fn new(State(state): State<AppState>, user: CurrentUser) -> AppResult<Response> {
    user.require(user.role.can_dispatch())?;
    render_form(&state, &user, None, &OrderForm::default(), &[])
}

pub async fn create(State(state): State<AppState>, user: CurrentUser, Form(form): Form<OrderForm>) -> AppResult<Response> {
    user.require(user.role.can_dispatch())?;
    let created = {
        let conn = state.db();
        let tariff = state.tariff.get(&conn)?;
        match form.validate(&tariff) {
            Ok(input) => Ok(orders::create(&conn, &tariff, &input, &user.display_name)?),
            Err(errors) => Err(errors),
        }
    };
    match created {
        Ok(id) => {
            let reference = orders::reference_for(id);
            state.sessions.flash(&user.token, "success", format!("{reference} created. It is now waiting for a rider."));
            Ok(Redirect::to(&format!("/orders/{id}")).into_response())
        }
        Err(errors) => render_form(&state, &user, None, &form, &errors),
    }
}

pub async fn show(State(state): State<AppState>, user: CurrentUser, Path(id): Path<i64>) -> AppResult<Response> {
    user.require(user.role != Role::Rider)?;
    let (order, timeline, fleet) = {
        let conn = state.db();
        let order = orders::get(&conn, id)?;
        let timeline = orders::timeline(&conn, id)?;
        let fleet = riders::list(&conn, clock::today())?;
        (order, timeline, fleet)
    };
    let fleet: Vec<_> = fleet.into_iter().filter(|r| r.rider.active).collect();
    let page = views::render(
        &state,
        Some(&user),
        "order_detail.html",
        context! { order => order, timeline => timeline, fleet => fleet },
    )?;
    Ok(page.into_response())
}

pub async fn edit(State(state): State<AppState>, user: CurrentUser, Path(id): Path<i64>) -> AppResult<Response> {
    user.require(user.role.can_dispatch())?;
    let order = {
        let conn = state.db();
        orders::get(&conn, id)?
    };
    if !order.is_editable() {
        state.sessions.flash(&user.token, "error", format!("{} can no longer be edited.", order.reference));
        return Ok(Redirect::to(&format!("/orders/{id}")).into_response());
    }
    render_form(&state, &user, Some(id), &OrderForm::from_order(&order), &[])
}

pub async fn update(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Form(form): Form<OrderForm>,
) -> AppResult<Response> {
    user.require(user.role.can_dispatch())?;
    let saved = {
        let conn = state.db();
        let tariff = state.tariff.get(&conn)?;
        match form.validate(&tariff) {
            Ok(input) => Ok(orders::update(&conn, &tariff, id, &input, &user.display_name)?),
            Err(errors) => Err(errors),
        }
    };
    match saved {
        Ok(()) => {
            state.sessions.flash(&user.token, "success", "Order details saved.");
            Ok(Redirect::to(&format!("/orders/{id}")).into_response())
        }
        Err(errors) => render_form(&state, &user, Some(id), &form, &errors),
    }
}

pub async fn assign(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Form(form): Form<AssignForm>,
) -> AppResult<Response> {
    user.require(user.role.can_dispatch())?;
    match dispatch::assign(&state, id, form.rider_id, &user.display_name).await {
        Ok(rider) => state.sessions.flash(&user.token, "success", format!("Assigned to {rider}. We have texted them.")),
        Err(err) => state.sessions.flash(&user.token, "error", err.to_string()),
    }
    Ok(Redirect::to(&format!("/orders/{id}")).into_response())
}

pub async fn change_status(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Form(form): Form<StatusForm>,
) -> AppResult<Response> {
    let action = Action::parse(&form.action).ok_or_else(|| AppError::BadRequest("Unknown action.".into()))?;
    let cash = if form.cash.trim().is_empty() {
        None
    } else {
        Some(money::parse_naira(&form.cash).ok_or_else(|| AppError::BadRequest("Cash collected must be an amount in naira.".into()))?)
    };
    let reference = {
        let conn = state.db();
        let order = orders::get(&conn, id)?;
        let allowed = match user.role {
            Role::Rider => user.rider_id.is_some() && order.rider_id == user.rider_id && action != Action::Cancel,
            role => role.can_dispatch(),
        };
        user.require(allowed)?;
        orders::transition(&conn, id, action, cash, &user.display_name)?;
        order.reference
    };
    state.sessions.flash(&user.token, "success", format!("{reference} marked as {}.", action.past_tense()));
    let back = if form.back.starts_with('/') { form.back } else { format!("/orders/{id}") };
    Ok(Redirect::to(&back).into_response())
}

/// Fee preview for the order form.
pub async fn quote(State(state): State<AppState>, _user: CurrentUser, Query(q): Query<QuoteQuery>) -> AppResult<Response> {
    let tariff = {
        let conn = state.db();
        state.tariff.get(&conn)?
    };
    let (Ok(pickup), Ok(dropoff), Some(km)) = (
        q.pickup_area_id.parse::<i64>(),
        q.dropoff_area_id.parse::<i64>(),
        parse_distance(&q.distance_km),
    ) else {
        return Ok(Json(json!({ "ok": false, "error": "Pick both areas and enter the distance." })).into_response());
    };
    let body = match tariff.quote(pickup, dropoff, km, q.express.is_some()) {
        Ok(quote) => json!({
            "ok": true,
            "total_kobo": quote.total_kobo,
            "total": money::format_naira(quote.total_kobo),
            "zone": quote.zone_name,
            "band": quote.band_label,
            "bridge": quote.bridge_fee_kobo > 0,
        }),
        Err(err) => json!({ "ok": false, "error": err.to_string() }),
    };
    Ok(Json(body).into_response())
}

fn render_form(state: &AppState, user: &CurrentUser, id: Option<i64>, form: &OrderForm, errors: &[String]) -> AppResult<Response> {
    let areas = {
        let conn = state.db();
        state.tariff.get(&conn)?.areas.clone()
    };
    let page = views::render(
        state,
        Some(user),
        "order_form.html",
        context! { form => form, areas => areas, errors => errors, order_id => id },
    )?;
    Ok(page.into_response())
}
