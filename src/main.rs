use std::net::SocketAddr;

use dispatchly::{config::Config, db, routes, seed, state::AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,dispatchly=info")),
        )
        .init();

    let config = Config::from_env();
    let conn = db::open(&config.db_path).expect("could not open the database");
    if config.reseed || seed::is_empty(&conn).expect("could not inspect the database") {
        seed::run(&conn).expect("could not seed demo data");
        tracing::info!("seeded demo data into {}", config.db_path);
    }

    let state = AppState::new(conn, &config);
    let app = routes::router(state, &config);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("could not bind port");
    tracing::info!("Dispatchly listening on http://localhost:{}", config.port);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("server error");
}
