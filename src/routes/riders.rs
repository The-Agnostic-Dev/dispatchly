use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use minijinja::context;

use crate::{
    auth::{CurrentUser, Role},
    clock,
    error::{AppError, AppResult},
    services::riders,
    state::AppState,
    views,
};

pub async fn index(State(state): State<AppState>, user: CurrentUser) -> AppResult<Response> {
    user.require(user.role != Role::Rider)?;
    let fleet = {
        let conn = state.db();
        riders::list(&conn, clock::today())?
    };
    Ok(views::render(&state, Some(&user), "riders.html", context! { fleet => fleet })?.into_response())
}

pub async fn show(State(state): State<AppState>, user: CurrentUser, Path(id): Path<i64>) -> AppResult<Response> {
    user.require(user.role != Role::Rider)?;
    let (rider, current, recent, cash) = {
        let conn = state.db();
        let rider = riders::get(&conn, id)?;
        let (current, recent) = riders::jobs(&conn, id)?;
        let cash = riders::cash_on_hand(&conn, id)?;
        (rider, current, recent, cash)
    };
    let page = views::render(
        &state,
        Some(&user),
        "rider_detail.html",
        context! { rider => rider, current => current, recent => recent, cash_on_hand => cash },
    )?;
    Ok(page.into_response())
}

/// The rider's own phone view: current jobs with the buttons they need on the road.
pub async fn my_jobs(State(state): State<AppState>, user: CurrentUser) -> AppResult<Response> {
    let rider_id = user.rider_id.ok_or(AppError::Forbidden)?;
    let (rider, current, recent, cash) = {
        let conn = state.db();
        let rider = riders::get(&conn, rider_id)?;
        let (current, recent) = riders::jobs(&conn, rider_id)?;
        let cash = riders::cash_on_hand(&conn, rider_id)?;
        (rider, current, recent, cash)
    };
    let page = views::render(
        &state,
        Some(&user),
        "my_jobs.html",
        context! { rider => rider, current => current, recent => recent, cash_on_hand => cash },
    )?;
    Ok(page.into_response())
}
