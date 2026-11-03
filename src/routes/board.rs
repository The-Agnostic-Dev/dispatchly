use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use minijinja::context;

use crate::{
    auth::{CurrentUser, Role},
    clock,
    error::AppResult,
    forms::SearchQuery,
    services::board,
    state::AppState,
    views,
};

pub async fn show(State(state): State<AppState>, user: CurrentUser, Query(search): Query<SearchQuery>) -> AppResult<Response> {
    if user.role == Role::Rider {
        return Ok(Redirect::to("/my-jobs").into_response());
    }
    let board = {
        let conn = state.db();
        board::load(&conn, clock::today(), &search.q)?
    };
    let page = views::render(
        &state,
        Some(&user),
        "board.html",
        context! { board => board, today_label => clock::day_label(clock::today()) },
    )?;
    Ok(page.into_response())
}
