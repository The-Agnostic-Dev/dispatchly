use axum::{
    Form,
    extract::State,
    http::{HeaderMap, header},
    response::{IntoResponse, Redirect, Response},
};
use minijinja::context;

use crate::{
    auth::{self, SESSION_COOKIE},
    error::AppResult,
    forms::LoginForm,
    state::AppState,
    views,
};

pub async fn login_page(State(state): State<AppState>) -> AppResult<Response> {
    Ok(views::render(&state, None, "login.html", context! { error => (), username => "" })?.into_response())
}

pub async fn login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> AppResult<Response> {
    let user = {
        let conn = state.db();
        auth::authenticate(&conn, &form.username, &form.password)?
    };
    let Some(user) = user else {
        let page = views::render(
            &state,
            None,
            "login.html",
            context! { error => "That username and password do not match.", username => form.username },
        )?;
        return Ok(page.into_response());
    };
    let home = user.role.home();
    let token = state.sessions.start(user);
    let cookie = format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax");
    Ok(([(header::SET_COOKIE, cookie)], Redirect::to(home)).into_response())
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = auth::cookie(&headers, SESSION_COOKIE) {
        state.sessions.end(&token);
    }
    let expired = format!("{SESSION_COOKIE}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax");
    ([(header::SET_COOKIE, expired)], Redirect::to("/login")).into_response()
}
