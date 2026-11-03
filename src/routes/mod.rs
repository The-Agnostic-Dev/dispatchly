use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use crate::{config::Config, state::AppState};

mod auth;
mod board;
mod orders;
mod remittance;
mod riders;
mod settings;

pub fn router(state: AppState, config: &Config) -> Router {
    Router::new()
        .route("/", get(board::show))
        .route("/healthz", get(|| async { "ok" }))
        .route("/login", get(auth::login_page).post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/orders", post(orders::create))
        .route("/orders/new", get(orders::new))
        .route("/orders/{id}", get(orders::show))
        .route("/orders/{id}/edit", get(orders::edit).post(orders::update))
        .route("/orders/{id}/assign", post(orders::assign))
        .route("/orders/{id}/status", post(orders::change_status))
        .route("/api/quote", get(orders::quote))
        .route("/riders", get(riders::index))
        .route("/riders/{id}", get(riders::show))
        .route("/my-jobs", get(riders::my_jobs))
        .route("/remittance", get(remittance::sheet))
        .route("/remittance/{rider_id}", get(remittance::rider).post(remittance::record))
        .route("/settings/tariff", get(settings::tariff))
        .route("/settings/tariff/zones", post(settings::save_zones))
        .route("/settings/tariff/bands", post(settings::save_bands))
        .nest_service("/static", ServeDir::new(&config.static_dir))
        .with_state(state)
}
