use std::path::Path;

use axum::response::Html;
use minijinja::{Environment, Value, context};

use crate::{auth::CurrentUser, clock, error::AppResult, money, state::AppState};

pub fn environment(dir: &Path) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader(dir));
    env.add_filter("naira", |kobo: i64| money::format_naira(kobo));
    env.add_filter("plain_naira", |kobo: i64| money::plain_naira(kobo));
    env.add_filter("km", |km: f64| format!("{km:.1} km"));
    env.add_filter("status_label", status_label);
    env.add_filter("time", |stored: String| clock::time_label(&stored));
    env.add_filter("datetime", |stored: String| clock::datetime_label(&stored));
    env
}

pub fn status_label(status: String) -> String {
    match status.as_str() {
        "pending" => "Waiting for rider",
        "assigned" => "Assigned",
        "picked_up" => "Picked up",
        "delivered" => "Delivered",
        "failed" => "Failed delivery",
        "cancelled" => "Cancelled",
        other => other,
    }
    .to_string()
}

/// Renders a page with the signed-in user and any pending flash message.
pub fn render(state: &AppState, user: Option<&CurrentUser>, name: &str, ctx: Value) -> AppResult<Html<String>> {
    let flash = user.and_then(|u| state.sessions.take_flash(&u.token));
    let template = state.templates.get_template(name)?;
    let html = template.render(context! {
        user => user,
        flash => flash,
        today => clock::today().to_string(),
        ..ctx
    })?;
    Ok(Html(html))
}
