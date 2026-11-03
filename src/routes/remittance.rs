use axum::{
    Form,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Duration;
use minijinja::context;

use crate::{
    auth::CurrentUser,
    clock,
    error::{AppError, AppResult},
    forms::{DayQuery, RemitForm},
    money,
    services::{remittance, riders},
    state::AppState,
    views,
};

fn chosen_day(query: &DayQuery) -> chrono::NaiveDate {
    query.date.as_deref().and_then(clock::parse_day).unwrap_or_else(clock::today)
}

pub async fn sheet(State(state): State<AppState>, user: CurrentUser, Query(query): Query<DayQuery>) -> AppResult<Response> {
    user.require(user.role.can_see_money())?;
    let day = chosen_day(&query);
    let rows = {
        let conn = state.db();
        remittance::day_sheet(&conn, day)?
    };
    let expected: i64 = rows.iter().map(|r| r.expected_kobo).sum();
    let received: i64 = rows.iter().map(|r| r.received_kobo).sum();
    let page = views::render(
        &state,
        Some(&user),
        "remittance.html",
        context! {
            rows => rows,
            day => day.to_string(),
            day_label => clock::day_label(day),
            prev_day => (day - Duration::days(1)).to_string(),
            next_day => (day + Duration::days(1)).to_string(),
            is_today => day == clock::today(),
            expected_kobo => expected,
            received_kobo => received,
        },
    )?;
    Ok(page.into_response())
}

pub async fn rider(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(rider_id): Path<i64>,
    Query(query): Query<DayQuery>,
) -> AppResult<Response> {
    user.require(user.role.can_see_money())?;
    let day = chosen_day(&query);
    let (rider, lines, handed_in) = {
        let conn = state.db();
        (
            riders::get(&conn, rider_id)?,
            remittance::rider_lines(&conn, rider_id, day)?,
            remittance::handed_in(&conn, rider_id, day)?,
        )
    };
    let expected: i64 = lines.iter().map(|l| l.collected_kobo).sum();
    let received: i64 = handed_in.iter().map(|r| r.amount_kobo).sum();
    let page = views::render(
        &state,
        Some(&user),
        "remittance_rider.html",
        context! {
            rider => rider,
            lines => lines,
            handed_in => handed_in,
            day => day.to_string(),
            day_label => clock::day_label(day),
            expected_kobo => expected,
            received_kobo => received,
            outstanding_kobo => expected - received,
        },
    )?;
    Ok(page.into_response())
}

pub async fn record(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(rider_id): Path<i64>,
    Form(form): Form<RemitForm>,
) -> AppResult<Response> {
    user.require(user.role.can_see_money())?;
    let day = clock::parse_day(&form.day).ok_or_else(|| AppError::BadRequest("Pick a valid day.".into()))?;
    let back = format!("/remittance/{rider_id}?date={day}");
    let amount = money::parse_naira(&form.amount).filter(|a| *a > 0);
    let Some(amount) = amount else {
        state.sessions.flash(&user.token, "error", "Enter the amount the rider handed in, like 25000.");
        return Ok(Redirect::to(&back).into_response());
    };
    {
        let conn = state.db();
        riders::get(&conn, rider_id)?;
        remittance::record(&conn, rider_id, day, amount, &user.display_name, form.note.trim())?;
    }
    state.sessions.flash(&user.token, "success", format!("Recorded {} received.", money::format_naira(amount)));
    Ok(Redirect::to(&back).into_response())
}
